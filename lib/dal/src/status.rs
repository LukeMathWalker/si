//! Status system that can send real time updates for a multi-step activity to external consumers,
//! such as the web frontend.

#![warn(missing_docs, clippy::missing_errors_doc, clippy::missing_panics_doc)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_data_pg::{PgError, PgPoolError};
use std::collections::{HashMap, HashSet};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    pk, schema::variant::leaves::LeafKind, standard_model::objects_from_rows, ActorView,
    AttributeValue, AttributeValueError, AttributeValueId, ChangeSetPk, Component, ComponentError,
    ComponentId, ComponentStatus, DalContext, ExternalProvider, ExternalProviderError,
    InternalProvider, InternalProviderError, Prop, PropError, PropId, SchemaVariant, SocketId,
    StandardModel, StandardModelError, Tenancy, Timestamp, TransactionsError, UserPk, WsEvent,
    WsEventError, WsEventResult, WsPayload,
};

const MODEL_TABLE: &str = "status_updates";

const LIST_ACTIVE: &str = include_str!("queries/status_update/list_active.sql");
const UPDATE_DATA: &str = include_str!("queries/status_update/update_data.sql");
const MARK_FINISHED: &str = include_str!("queries/status_update/mark_finished.sql");

/// A possible error that can be returned when working with a [`StatusUpdate`].
#[derive(Error, Debug)]
pub enum StatusUpdateError {
    /// When an attribute value metadata entry is not found
    #[error("attribute value metadata not found for: {0}")]
    AttributeValueMetadataNotFound(AttributeValueId),
    /// When a pg error is returned
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    /// When a pg pool error is returned
    #[error("pg pool error: {0}")]
    PgPool(#[source] Box<PgPoolError>),
    /// When a JSON serialize/deserialize error is returned
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    /// When a standard model error is returned
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    /// When a user is not found by id
    #[error("user not found with pk: {0}")]
    UserNotFound(UserPk),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
}

impl From<PgPoolError> for StatusUpdateError {
    fn from(value: PgPoolError) -> Self {
        Self::PgPool(Box::new(value))
    }
}

/// A useful [`Result`] alias when working with a [`StatusUpdate`].
pub type StatusUpdateResult<T> = Result<T, StatusUpdateError>;

pk!(
    /// A primary key for a [`StatusUpdate`].
    StatusUpdatePk
);

/// The internal state data of a [`StatusUpdate`].
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StatusUpdateData {
    actor: ActorView,
    dependent_values_metadata: HashMap<AttributeValueId, AttributeValueMetadata>,
    queued_dependent_value_ids: HashSet<AttributeValueId>,
    running_dependent_value_ids: HashSet<AttributeValueId>,
    completed_dependent_value_ids: HashSet<AttributeValueId>,
}

impl postgres_types::ToSql for StatusUpdateData {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        let json = serde_json::to_value(self)?;
        postgres_types::ToSql::to_sql(&json, ty, out)
    }

    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        ty == &postgres_types::Type::JSONB
    }

    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let json = serde_json::to_value(self)?;
        postgres_types::ToSql::to_sql(&json, ty, out)
    }
}

/// A `StatusUpdate` tracks the progress of a complex event which has more than one phase or step.
///
/// # Implementation Notes
///
/// A `StatusUpdate` lives outside of a normal [`DalContext`] database transaction. It behaves more
/// like a `HistoryEvent` or a general audit event.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StatusUpdate {
    /// The primary key
    pub pk: StatusUpdatePk,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    finished_at: Option<DateTime<Utc>>,
    change_set_pk: ChangeSetPk,
    /// The update data
    pub data: StatusUpdateData,
}

impl StatusUpdate {
    /// Creates and persists a new initialized `StatusUpdate`.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the datastore is unable to persist the new object.
    pub async fn new(ctx: &DalContext) -> StatusUpdateResult<Self> {
        let new_ctx = ctx.clone_with_new_transactions().await?;
        let ctx = &new_ctx;
        let actor = ActorView::from_history_actor(ctx, *ctx.history_actor()).await?;

        // This query explicitly uses its own connection to bypass/avoid a ctx's database
        // transaction--status updates live outside of transactions!
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM status_update_create_v1($1, $2, $3)",
                &[&ctx.visibility().change_set_pk, &actor, ctx.tenancy()],
            )
            .await?;

        new_ctx.commit().await?;

        let json: serde_json::Value = row.try_get("object")?;
        let object: Self = serde_json::from_value(json)?;
        Ok(object)
    }

    /// Returns `StatusUpdate`s for a changeset that are un-finished (i.e. "active").
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if there is a connection issue.
    pub async fn list_active(ctx: &DalContext) -> StatusUpdateResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_ACTIVE,
                &[&ctx.visibility().change_set_pk, ctx.tenancy()],
            )
            .await?;

        objects_from_rows(rows).map_err(Into::into)
    }

    /// Returns a map of all attribute value metadata, keyed by [`AttributeValueId`].
    pub fn dependent_values_metadata(&self) -> &HashMap<AttributeValueId, AttributeValueMetadata> {
        &self.data.dependent_values_metadata
    }

    /// Sets all dependent value metadata information and persists the update.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if there is a connection issue or if the update fails.
    pub async fn set_dependent_values_metadata(
        &mut self,
        ctx: &DalContext,
        dependent_values_metadata: HashMap<AttributeValueId, AttributeValueMetadata>,
    ) -> StatusUpdateResult<()> {
        self.data.dependent_values_metadata = dependent_values_metadata;
        self.persist_data_to_db(ctx).await
    }

    /// Returns a set of all currently queued [`AttributeValueId`s](AttributeValueId).
    pub fn queued_dependent_value_ids(&self) -> &HashSet<AttributeValueId> {
        &self.data.queued_dependent_value_ids
    }

    /// Sets all queued dependent value ids, persists the update, and returns a parallel `Vec` of
    /// [`AttributeValueMetadata`].
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if there is a connection issue or if the update fails.
    pub async fn set_queued_dependent_value_ids(
        &mut self,
        ctx: &DalContext,
        value_ids: Vec<AttributeValueId>,
    ) -> StatusUpdateResult<Vec<AttributeValueMetadata>> {
        self.data.queued_dependent_value_ids.extend(&value_ids);
        self.persist_data_to_db(ctx).await?;

        self.metadata_from_value_ids(value_ids)
    }

    /// Returns a set of all currently running [`AttributeValueId`s](AttributeValueId).
    pub fn running_dependent_value_ids(&self) -> &HashSet<AttributeValueId> {
        &self.data.running_dependent_value_ids
    }

    /// Sets all running dependent value ids, persists the update, and returns a parallel `Vec` of
    /// [`AttributeValueMetadata`].
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if there is a connection issue or if the update fails.
    pub async fn set_running_dependent_value_ids(
        &mut self,
        ctx: &DalContext,
        value_ids: Vec<AttributeValueId>,
    ) -> StatusUpdateResult<Vec<AttributeValueMetadata>> {
        for value_id in value_ids.iter() {
            self.data.queued_dependent_value_ids.remove(value_id);
        }
        self.data.running_dependent_value_ids.extend(&value_ids);
        self.persist_data_to_db(ctx).await?;

        self.metadata_from_value_ids(value_ids)
    }

    /// Returns a set of all currently completed [`AttributeValueId`s](AttributeValueId).
    pub fn completed_dependent_value_ids(&self) -> &HashSet<AttributeValueId> {
        &self.data.completed_dependent_value_ids
    }

    /// Sets all completed dependent value ids, persists the update, and returns a parallel `Vec` of
    /// [`AttributeValueMetadata`].
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if there is a connection issue or if the update fails.
    pub async fn set_completed_dependent_value_ids(
        &mut self,
        ctx: &DalContext,
        value_ids: Vec<AttributeValueId>,
    ) -> StatusUpdateResult<Vec<AttributeValueMetadata>> {
        for value_id in value_ids.iter() {
            self.data.running_dependent_value_ids.remove(value_id);
        }
        self.data.completed_dependent_value_ids.extend(&value_ids);
        self.persist_data_to_db(ctx).await?;

        self.metadata_from_value_ids(value_ids)
    }

    /// Marks the status update as finished and persists the update.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if there is a connection issue or if the update fails.
    pub async fn finish(&mut self, ctx: &DalContext) -> StatusUpdateResult<()> {
        let row = ctx
            .txns()
            .pg()
            .query_one(MARK_FINISHED, &[&self.pk])
            .await?;

        let updated_at = row.try_get("updated_at").map_err(|_| {
            StandardModelError::ModelMissing(MODEL_TABLE.to_string(), self.pk.to_string())
        })?;
        let finished_at = row.try_get("finished_at").map_err(|_| {
            StandardModelError::ModelMissing(MODEL_TABLE.to_string(), self.pk.to_string())
        })?;
        self.timestamp.updated_at = updated_at;
        self.finished_at = Some(finished_at);

        Ok(())
    }

    async fn persist_data_to_db(&mut self, ctx: &DalContext) -> StatusUpdateResult<()> {
        // This query explicitly uses its own connection to bypass/avoid a ctx's database
        // transaction--status updates live outside of transactions!
        let row = ctx
            .txns()
            .pg()
            .query_one(UPDATE_DATA, &[&self.pk, &self.data])
            .await?;

        let updated_at = row.try_get("updated_at").map_err(|_| {
            StandardModelError::ModelMissing(MODEL_TABLE.to_string(), self.pk.to_string())
        })?;
        self.timestamp.updated_at = updated_at;

        Ok(())
    }

    fn metadata_from_value_ids(
        &self,
        value_ids: Vec<AttributeValueId>,
    ) -> StatusUpdateResult<Vec<AttributeValueMetadata>> {
        value_ids
            .iter()
            .map(|id| {
                self.dependent_values_metadata()
                    .get(id)
                    .ok_or(StatusUpdateError::AttributeValueMetadataNotFound(*id))
                    .map(|e| *e)
            })
            .collect::<Result<Vec<_>, _>>()
    }
}

/// The state of a status update message.
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StatusMessageState {
    /// A status update has started
    StatusStarted,
    /// A message which has newly queued entries
    Queued,
    /// A message which has newly running entries
    Running,
    /// A message which has newly completed entries
    Completed,
    /// A status update has finished
    StatusFinished,
}

/// A status message which encapsulates a new status for some subset of entries.
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StatusMessage {
    pk: StatusUpdatePk,
    status: StatusMessageState,
    values: Vec<AttributeValueMetadata>,
}

/// A representation of the kind of attribute value that is being processed.
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq, Copy, Hash)]
#[serde(rename_all = "camelCase", tag = "kind", content = "id")]
enum AttributeValueKind {
    /// Represents a raw attribute value with associated `PropId`
    Attribute(PropId),
    /// Represents a value resulting from a code generation function
    CodeGen,
    /// Represents a value resulting from a qualification function
    Qualification,
    /// Represents a value that is internal to a component
    Internal,
    /// Represents a value used as an input socket with associated `SocketId`
    InputSocket(SocketId),
    /// Represents a value used as an output socket with associated `SocketId`
    OutputSocket(SocketId),
}

/// A computed set of metadata relating to an [`AttributeValue`].
#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AttributeValueMetadata {
    value_id: AttributeValueId,
    component_id: ComponentId,
    value_kind: AttributeValueKind,
}

impl AttributeValueMetadata {
    fn new(
        value_id: AttributeValueId,
        component_id: ComponentId,
        value_kind: AttributeValueKind,
    ) -> Self {
        Self {
            value_id,
            component_id,
            value_kind,
        }
    }
}

/// A possible error that can be returned when working with a [`StatusUpdater`].
#[derive(Debug, Error)]
pub enum StatusUpdaterError {
    /// When an attribute value metadata fails to be created
    #[error("attribute value metadata error {0}")]
    AttributeValueMetadata(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    /// When a component's status fails to update
    #[error("component error {0}")]
    Component(#[from] ComponentError),
    /// When a realtime update fails to send
    #[error("error publishing realtime update {0}")]
    PublishRealtime(#[from] WsEventError),
    /// When a status update error is returned
    #[error(transparent)]
    StatusUpdate(#[from] StatusUpdateError),
    /// When there are unprocessed values remaining once an update has completed
    #[error("unprocessed values remain upon completion: {0:?}")]
    UnprocessedValuesRemaining(Vec<AttributeValueId>),
    /// Generic error in status updater
    #[error("status update error: {0}")]
    GenericError(String),
    /// When a JSON serialize/deserialize error is returned
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    /// When a [NATS](https://nats.io) error is encountered
    #[error("nats error: {0}")]
    NatsError(#[from] si_data_nats::Error),
    /// When a transaction error occurs
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
}

impl StatusUpdaterError {
    /// Creates a new `StatusUpdateError` when creating an [`AttributeValueMetadata`].
    pub fn metadata(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::AttributeValueMetadata(Box::new(source))
    }
}

async fn get_ctx(
    ctx: &DalContext,
    single_transaction: bool,
) -> Result<DalContext, StatusUpdaterError> {
    Ok(if single_transaction {
        ctx.clone()
    } else {
        ctx.clone_with_new_transactions().await?
    })
}

/// Tracks and maintains the persisted and realtime state of a [`StatusUpdate`].
#[derive(Clone, Debug)]
pub struct StatusUpdater {
    model: StatusUpdate,
    single_transaction: bool,
}

impl StatusUpdater {
    /// Intializes a `StatusUpdater` with an underlying [`StatusUpdate`].
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the datastore is unable to persist the new object.
    pub async fn initialize(
        ctx: &DalContext,
        single_transaction: bool,
    ) -> Result<Self, StatusUpdaterError> {
        dbg!("initializing status updater");
        let local_ctx = get_ctx(ctx, single_transaction).await?;
        let ctx = &local_ctx;
        let model = StatusUpdate::new(ctx).await?;

        Self::publish_immediately(
            ctx,
            WsEvent::status_update(ctx, model.pk, StatusMessageState::StatusStarted, vec![])
                .await?,
        )
        .await?;

        if !single_transaction {
            local_ctx.commit().await?;
        }

        Ok(Self {
            model,
            single_transaction,
        })
    }

    pub async fn get_ctx(&self, ctx: &DalContext) -> Result<DalContext, StatusUpdaterError> {
        get_ctx(ctx, self.single_transaction).await
    }

    /// Updates the [`StatusUpdate`] with a new set of queued values, represented with their
    /// [`AttributeValueId`s](AttributeValueId).
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the datastore is unable to update the status update.
    pub async fn values_queued(
        &mut self,
        ctx: &DalContext,
        value_ids: Vec<AttributeValueId>,
    ) -> Result<(), StatusUpdaterError> {
        let local_ctx = self.get_ctx(ctx).await?;
        let ctx = &local_ctx;
        let now = std::time::Instant::now();

        let mut dependent_values_metadata: HashMap<AttributeValueId, AttributeValueMetadata> =
            HashMap::new();

        for value_id in value_ids {
            let attribute_value = AttributeValue::get_by_id(ctx, &value_id)
                .await
                .map_err(StatusUpdaterError::metadata)?
                .ok_or_else(|| {
                    StatusUpdaterError::metadata(AttributeValueError::NotFound(
                        value_id,
                        *ctx.visibility(),
                    ))
                })?;
            let component_id = attribute_value.context.component_id();

            let mut value_kind;

            // does this value look like an output socket?
            if !component_id.is_some() {
                // this should only happen when migrating builtins, we'll just skip this whole case
                value_kind = AttributeValueKind::Internal;
            } else if attribute_value
                .context
                .is_least_specific_field_kind_external_provider()
                .map_err(StatusUpdaterError::metadata)?
            {
                let external_provider = ExternalProvider::get_by_id(
                    ctx,
                    &attribute_value.context.external_provider_id(),
                )
                .await
                .map_err(StatusUpdaterError::metadata)?
                .ok_or_else(|| {
                    StatusUpdaterError::metadata(ExternalProviderError::NotFound(
                        attribute_value.context.external_provider_id(),
                    ))
                })?;
                let socket = external_provider
                    .sockets(ctx)
                    .await
                    .map_err(StatusUpdaterError::metadata)?
                    .pop()
                    .expect("no sockets in vec");
                value_kind = AttributeValueKind::OutputSocket(*socket.id());

                // does this value look like an input socket?
            } else if attribute_value
                .context
                .is_least_specific_field_kind_internal_provider()
                .map_err(StatusUpdaterError::metadata)?
            {
                let internal_provider = InternalProvider::get_by_id(
                    ctx,
                    &attribute_value.context.internal_provider_id(),
                )
                .await
                .map_err(StatusUpdaterError::metadata)?
                .ok_or_else(|| {
                    StatusUpdaterError::metadata(InternalProviderError::NotFound(
                        attribute_value.context.internal_provider_id(),
                    ))
                })?;
                if internal_provider.prop_id().is_none() {
                    let socket = internal_provider
                        .sockets(ctx)
                        .await
                        .map_err(StatusUpdaterError::metadata)?
                        .pop()
                        .expect("no sockets in vec");
                    value_kind = AttributeValueKind::InputSocket(*socket.id());
                } else {
                    value_kind = AttributeValueKind::Internal;
                }

                // does this value correspond to a code generation function?
            } else if attribute_value.context.prop_id().is_some() {
                value_kind = AttributeValueKind::Attribute(attribute_value.context.prop_id());
                let ctx_deleted = &ctx.clone_with_delete_visibility();
                let component =
                    Component::get_by_id(ctx_deleted, &attribute_value.context.component_id())
                        .await
                        .map_err(StatusUpdaterError::metadata)?
                        .ok_or_else(|| {
                            StatusUpdaterError::GenericError(format!(
                                "Unable to find component {}",
                                attribute_value.context.component_id()
                            ))
                        })?;

                let schema_variant = component
                    .schema_variant(ctx_deleted)
                    .await
                    .map_err(StatusUpdaterError::metadata)?
                    .ok_or_else(|| {
                        StatusUpdaterError::GenericError(format!(
                            "Unable to load schema variant from component {component_id}"
                        ))
                    })?;

                // SchemaVariant::find_code_implicit_internal_provider(ctx, schema_variant_id)
                let code_item_prop = SchemaVariant::find_leaf_item_prop(
                    ctx,
                    *schema_variant.id(),
                    LeafKind::CodeGeneration,
                )
                .await
                .map_err(StatusUpdaterError::metadata)?;

                let prop = Prop::get_by_id(ctx, &attribute_value.context.prop_id())
                    .await
                    .map_err(StatusUpdaterError::metadata)?
                    .ok_or_else(|| {
                        StatusUpdaterError::metadata(PropError::NotFound(
                            attribute_value.context.prop_id(),
                            *ctx.visibility(),
                        ))
                    })?;
                if prop.id() == code_item_prop.id() {
                    value_kind = AttributeValueKind::CodeGen;
                } else if let Some(parent_prop) = prop
                    .parent_prop(ctx)
                    .await
                    .map_err(StatusUpdaterError::metadata)?
                {
                    if parent_prop.id() == code_item_prop.id() {
                        value_kind = AttributeValueKind::CodeGen;
                    } else if let Some(grandparent_prop) = parent_prop
                        .parent_prop(ctx)
                        .await
                        .map_err(StatusUpdaterError::metadata)?
                    {
                        if grandparent_prop.id() == code_item_prop.id() {
                            value_kind = AttributeValueKind::CodeGen;
                        }
                    }
                }
            } else {
                unreachable!("unexpectedly found a value that is not internal but has no prop id")
            }

            dependent_values_metadata.insert(
                value_id,
                AttributeValueMetadata::new(value_id, component_id, value_kind),
            );
        }

        self.model
            .set_dependent_values_metadata(ctx, dependent_values_metadata)
            .await?;

        let queued_values = self
            .model
            .set_queued_dependent_value_ids(
                ctx,
                self.model
                    .dependent_values_metadata()
                    .keys()
                    .copied()
                    .collect(),
            )
            .await?;

        let elapsed_time = now.elapsed();
        info!("StatusUpdater.values_queued took {:?}", elapsed_time);

        Self::publish_immediately(
            ctx,
            WsEvent::status_update(
                ctx,
                self.model.pk,
                StatusMessageState::Queued,
                queued_values.into_iter().collect(),
            )
            .await?,
        )
        .await?;

        if !self.single_transaction {
            local_ctx.commit().await?;
        }

        Ok(())
    }

    /// Updates the [`StatusUpdate`] with a new set of running values, represented with their
    /// [`AttributeValueId`s](AttributeValueId).
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the datastore is unable to update the status update.
    pub async fn values_running(
        &mut self,
        ctx: &DalContext,
        value_ids: Vec<AttributeValueId>,
    ) -> Result<(), StatusUpdaterError> {
        let local_ctx = self.get_ctx(ctx).await?;
        let ctx = &local_ctx;
        let running_values = self
            .model
            .set_running_dependent_value_ids(ctx, value_ids)
            .await?;

        Self::publish_immediately(
            ctx,
            WsEvent::status_update(
                ctx,
                self.model.pk,
                StatusMessageState::Running,
                running_values,
            )
            .await?,
        )
        .await?;

        if !self.single_transaction {
            local_ctx.commit().await?;
        }

        Ok(())
    }

    /// Updates the [`StatusUpdate`] with a new set of completed values, represented with their
    /// [`AttributeValueId`s](AttributeValueId).
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the datastore is unable to update the status update.
    pub async fn values_completed(
        &mut self,
        ctx: &DalContext,
        value_ids: Vec<AttributeValueId>,
    ) -> Result<(), StatusUpdaterError> {
        let local_ctx = self.get_ctx(ctx).await?;
        let ctx = &local_ctx;

        let completed_values = self
            .model
            .set_completed_dependent_value_ids(ctx, value_ids)
            .await?;

        // Record that the component was "updated" for every distinct component in the collection
        // of attribute values. Note that this call will intentionally use the `ctx`'s txn as we
        // only want to record these updates if the running txn gets committed.
        for component_id in completed_values
            .iter()
            .map(|metadata| metadata.component_id)
            .collect::<HashSet<_>>()
            .into_iter()
            .filter(|component_id| component_id.is_some())
        {
            ComponentStatus::record_update_by_id(ctx, component_id).await?;
        }

        WsEvent::status_update(
            ctx,
            self.model.pk,
            StatusMessageState::Completed,
            completed_values,
        )
        .await?
        .publish_on_commit(ctx)
        .await?;

        if !self.single_transaction {
            local_ctx.commit().await?;
        }

        Ok(())
    }

    /// Marks the [`StatusUpdate`] as finished, and ensures that there are no unprocessed values.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if there are unprocessed values.
    pub async fn finish(mut self, ctx: &DalContext) -> Result<(), StatusUpdaterError> {
        let local_ctx = self.get_ctx(ctx).await?;
        let ctx = &local_ctx;

        self.model.finish(ctx).await?;

        let all_value_ids = self
            .model
            .dependent_values_metadata()
            .keys()
            .copied()
            .collect::<HashSet<AttributeValueId>>();
        let completed_value_ids = self.model.completed_dependent_value_ids();
        if &all_value_ids != completed_value_ids {
            return Err(StatusUpdaterError::UnprocessedValuesRemaining(
                all_value_ids
                    .difference(completed_value_ids)
                    .copied()
                    .collect(),
            ));
        }

        if !self.model.queued_dependent_value_ids().is_empty() {
            return Err(StatusUpdaterError::UnprocessedValuesRemaining(
                self.model
                    .queued_dependent_value_ids()
                    .iter()
                    .copied()
                    .collect(),
            ));
        }
        if !self.model.running_dependent_value_ids().is_empty() {
            return Err(StatusUpdaterError::UnprocessedValuesRemaining(
                self.model
                    .running_dependent_value_ids()
                    .iter()
                    .copied()
                    .collect(),
            ));
        }

        Self::publish_immediately(
            ctx,
            WsEvent::status_update(
                ctx,
                self.model.pk,
                StatusMessageState::StatusFinished,
                vec![],
            )
            .await?,
        )
        .await?;

        if !self.single_transaction {
            local_ctx.commit().await?;
        }

        Ok(())
    }

    /// Publish a [`WsEvent`](crate::WsEvent) immediately.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the [`event`](crate::WsEvent) could not be published or the payload
    /// could not be serialized.
    ///
    /// # Notes
    ///
    /// This should only be done unless the caller is _certain_ that the [`event`](crate::WsEvent)
    /// should be published immediately. If unsure, use
    /// [`WsEvent::publish`](crate::WsEvent::publish_on_commit).
    ///
    /// This method requires an owned [`WsEvent`](crate::WsEvent), despite it not needing to,
    /// because [`events`](crate::WsEvent) should likely not be reused.
    async fn publish_immediately(
        ctx: &DalContext,
        ws_event: WsEvent,
    ) -> Result<(), StatusUpdaterError> {
        // TODO(nick,fletcher): this method should be deleted once status updater is fully moved
        // to the status receiver because the status receiver should have its own ability to
        // "immediately publish" events.
        let subject = format!("si.workspace_pk.{}.event", ws_event.workspace_pk());
        let msg_bytes = serde_json::to_vec(&ws_event)?;
        ctx.nats_conn().publish(subject, msg_bytes).await?;
        Ok(())
    }
}

impl WsEvent {
    /// Creates a new `WsEvent` for a [`StatusUpdate`].
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if no user exists for a user pk or if there is a connection issue with the
    /// database.
    pub async fn status_update(
        ctx: &DalContext,
        pk: StatusUpdatePk,
        status: StatusMessageState,
        values: Vec<AttributeValueMetadata>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::StatusUpdate(StatusMessage { pk, status, values }),
        )
        .await
    }
}
