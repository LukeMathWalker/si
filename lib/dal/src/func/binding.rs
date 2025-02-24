use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;
use veritech_client::{OutputStream, ResolverFunctionComponent};

use crate::func::execution::FuncExecutionPk;
use crate::FuncError;
use crate::{
    func::backend::{
        array::FuncBackendArray,
        boolean::FuncBackendBoolean,
        diff::FuncBackendDiff,
        identity::FuncBackendIdentity,
        integer::FuncBackendInteger,
        js_action::FuncBackendJsAction,
        js_attribute::{FuncBackendJsAttribute, FuncBackendJsAttributeArgs},
        js_reconciliation::FuncBackendJsReconciliation,
        js_schema_variant_definition::FuncBackendJsSchemaVariantDefinition,
        js_validation::FuncBackendJsValidation,
        map::FuncBackendMap,
        object::FuncBackendObject,
        string::FuncBackendString,
        validation::FuncBackendValidation,
        FuncBackend, FuncDispatch, FuncDispatchContext,
    },
    TransactionsError, WsEvent, WsEventError, WsEventResult, WsPayload,
};
use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_belongs_to,
    Func, FuncBackendError, FuncBackendKind, HistoryEventError, StandardModel, StandardModelError,
    Timestamp, Visibility,
};
use crate::{DalContext, Tenancy};

use super::{
    binding_return_value::{FuncBindingReturnValue, FuncBindingReturnValueError},
    execution::{FuncExecution, FuncExecutionError},
    FuncId,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncBindingError {
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func backend error: {0}")]
    FuncBackend(#[from] FuncBackendError),
    #[error(
        "function execution result failure: kind={kind}, message={message}, backend={backend}"
    )]
    FuncBackendResultFailure {
        kind: String,
        message: String,
        backend: String,
    },
    #[error("func backend return value error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("func execution tracking error: {0}")]
    FuncExecutionError(#[from] FuncExecutionError),
    #[error("unable to retrieve func for func binding: {0:?}")]
    FuncNotFound(FuncBindingPk),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("unable to retrieve func for func binding: {0:?}")]
    JsFuncNotFound(FuncBindingPk),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("func binding not found: {0}")]
    NotFound(FuncBindingId),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type FuncBindingResult<T> = Result<T, FuncBindingError>;

pk!(FuncBindingPk);
pk!(FuncBindingId);

/// A [`FuncBinding`] binds an execution context (including arguments) to a [`Func`](crate::Func),
/// so that it can be executed.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FuncBinding {
    pk: FuncBindingPk,
    id: FuncBindingId,
    args: serde_json::Value,
    backend_kind: FuncBackendKind,
    code_sha256: String,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: FuncBinding,
    pk: FuncBindingPk,
    id: FuncBindingId,
    table_name: "func_bindings",
    history_event_label_base: "func_binding",
    history_event_message_name: "Func Binding"
}

impl FuncBinding {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        args: serde_json::Value,
        func_id: FuncId,
        backend_kind: FuncBackendKind,
    ) -> FuncBindingResult<Self> {
        let func = Func::get_by_id(ctx, &func_id)
            .await?
            .ok_or(FuncBindingError::FuncNotFound(FuncBindingPk::NONE))?;

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM func_binding_create_v1($1, $2, $3, $4, $5, $6)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &args,
                    &func_id,
                    &backend_kind.as_ref(),
                    &func.code_sha256(),
                ],
            )
            .await?;
        let object: FuncBinding = standard_model::finish_create_from_row(ctx, row).await?;
        object.set_func(ctx, &func_id).await?;
        Ok(object)
    }

    pub async fn create_with_existing_value(
        ctx: &DalContext,
        args: serde_json::Value,
        value: Option<serde_json::Value>,
        func_id: FuncId,
    ) -> FuncBindingResult<(Self, FuncBindingReturnValue)> {
        let func = Func::get_by_id(ctx, &func_id)
            .await?
            .ok_or(FuncError::NotFound(func_id))?;
        let func_binding = Self::new(ctx, args, func_id, func.backend_kind).await?;

        let func_binding_return_value = FuncBindingReturnValue::new(
            ctx,
            value.clone(),
            value,
            func_id,
            *func_binding.id(),
            FuncExecutionPk::NONE,
        )
        .await?;

        Ok((func_binding, func_binding_return_value))
    }

    /// Runs [`Self::new()`] and executes.
    ///
    /// Use this function if you would like to receive the
    /// [`FuncBindingReturnValue`](crate::FuncBindingReturnValue) for a given
    /// [`FuncId`](crate::Func) and [`args`](serde_json::Value).
    pub async fn create_and_execute(
        ctx: &DalContext,
        args: serde_json::Value,
        func_id: FuncId,
    ) -> FuncBindingResult<(Self, FuncBindingReturnValue)> {
        let func = Func::get_by_id(ctx, &func_id)
            .await?
            .ok_or(FuncError::NotFound(func_id))?;
        let func_binding = Self::new(ctx, args, func_id, func.backend_kind).await?;

        let func_binding_return_value: FuncBindingReturnValue = func_binding.execute(ctx).await?;

        Ok((func_binding, func_binding_return_value))
    }

    standard_model_accessor!(args, PlainJson<JsonValue>, FuncBindingResult);
    standard_model_accessor!(backend_kind, Enum(FuncBackendKind), FuncBindingResult);
    standard_model_accessor!(code_sha256, String, FuncBindingResult);
    standard_model_belongs_to!(
        lookup_fn: func,
        set_fn: set_func,
        unset_fn: unset_func,
        table: "func_binding_belongs_to_func",
        model_table: "funcs",
        belongs_to_id: FuncId,
        returns: Func,
        result: FuncBindingResult,
    );

    // For a given [`FuncBinding`](Self), execute using veritech.
    pub async fn execute(&self, ctx: &DalContext) -> FuncBindingResult<FuncBindingReturnValue> {
        let (func, execution, context, mut rx) = self.prepare_execution(ctx).await?;
        let value = self.execute_critical_section(func.clone(), context).await?;

        let mut output = Vec::new();
        while let Some(output_stream) = rx.recv().await {
            output.push(output_stream);
        }

        self.postprocess_execution(ctx, output, &func, value, execution)
            .await
    }

    /// Perform function execution to veritech for a given [`Func`](crate::Func) and
    /// [`FuncDispatchContext`](crate::func::backend::FuncDispatchContext).
    pub async fn execute_critical_section(
        &self,
        func: Func,
        context: FuncDispatchContext,
    ) -> FuncBindingResult<(Option<serde_json::Value>, Option<serde_json::Value>)> {
        // TODO: encrypt components
        let execution_result = match self.backend_kind() {
            FuncBackendKind::JsValidation => {
                FuncBackendJsValidation::create_and_execute(context, &func, &self.args).await
            }
            FuncBackendKind::JsAction => {
                FuncBackendJsAction::create_and_execute(context, &func, &self.args).await
            }
            FuncBackendKind::JsReconciliation => {
                FuncBackendJsReconciliation::create_and_execute(context, &func, &self.args).await
            }
            FuncBackendKind::JsAttribute => {
                let args = FuncBackendJsAttributeArgs {
                    component: ResolverFunctionComponent {
                        data: veritech_client::ComponentView {
                            properties: self.args.clone(),
                            ..Default::default()
                        },
                        parents: Vec::new(),
                    },
                    response_type: (*func.backend_response_type()).into(),
                };
                FuncBackendJsAttribute::create_and_execute(
                    context,
                    &func,
                    &serde_json::to_value(args)?,
                )
                .await
            }
            FuncBackendKind::JsSchemaVariantDefinition => {
                FuncBackendJsSchemaVariantDefinition::create_and_execute(
                    context,
                    &func,
                    &serde_json::Value::Null,
                )
                .await
            }
            FuncBackendKind::Array => FuncBackendArray::create_and_execute(&self.args).await,
            FuncBackendKind::Boolean => FuncBackendBoolean::create_and_execute(&self.args).await,
            FuncBackendKind::Identity => FuncBackendIdentity::create_and_execute(&self.args).await,
            FuncBackendKind::Diff => FuncBackendDiff::create_and_execute(&self.args).await,
            FuncBackendKind::Integer => FuncBackendInteger::create_and_execute(&self.args).await,
            FuncBackendKind::Map => FuncBackendMap::create_and_execute(&self.args).await,
            FuncBackendKind::Object => FuncBackendObject::create_and_execute(&self.args).await,
            FuncBackendKind::String => FuncBackendString::create_and_execute(&self.args).await,
            FuncBackendKind::Unset => Ok((None, None)),
            FuncBackendKind::Validation => {
                FuncBackendValidation::create_and_execute(&self.args).await
            }
        };

        match execution_result {
            Ok(value) => Ok(value),
            Err(FuncBackendError::ResultFailure {
                kind,
                message,
                backend,
            }) => Err(FuncBindingError::FuncBackendResultFailure {
                kind,
                message,
                backend,
            }),
            Err(err) => Err(err)?,
        }
    }

    pub async fn postprocess_execution(
        &self,
        ctx: &DalContext,
        output_stream: Vec<OutputStream>,
        func: &Func,
        (unprocessed_value, processed_value): (
            Option<serde_json::Value>,
            Option<serde_json::Value>,
        ),
        mut execution: FuncExecution,
    ) -> FuncBindingResult<FuncBindingReturnValue> {
        execution.set_output_stream(ctx, output_stream).await?;

        let func_binding_return_value = FuncBindingReturnValue::new(
            ctx,
            unprocessed_value,
            processed_value,
            *func.id(),
            self.id,
            execution.pk(),
        )
        .await?;

        execution
            .process_return_value(ctx, &func_binding_return_value)
            .await?;
        execution
            .set_state(ctx, super::execution::FuncExecutionState::Success)
            .await?;

        Ok(func_binding_return_value)
    }

    pub async fn prepare_execution(
        &self,
        ctx: &DalContext,
    ) -> FuncBindingResult<(
        Func,
        FuncExecution,
        FuncDispatchContext,
        mpsc::Receiver<OutputStream>,
    )> {
        let func: Func = self
            .func(ctx)
            .await?
            .ok_or(FuncBindingError::FuncNotFound(self.pk))?;

        let mut execution = FuncExecution::new(ctx, &func, self).await?;

        match self.backend_kind() {
            FuncBackendKind::Array
            | FuncBackendKind::Boolean
            | FuncBackendKind::Identity
            | FuncBackendKind::Diff
            | FuncBackendKind::Integer
            | FuncBackendKind::Map
            | FuncBackendKind::Object
            | FuncBackendKind::String
            | FuncBackendKind::Unset
            | FuncBackendKind::Validation => {}

            FuncBackendKind::JsAction
            | FuncBackendKind::JsAttribute
            | FuncBackendKind::JsReconciliation
            | FuncBackendKind::JsSchemaVariantDefinition
            | FuncBackendKind::JsValidation => {
                execution
                    .set_state(ctx, super::execution::FuncExecutionState::Dispatch)
                    .await?;
            }
        }

        // NOTE(nick,wendy): why is the state is set to Run immediately after it is set to
        // Dispatch a few lines above?
        execution
            .set_state(ctx, super::execution::FuncExecutionState::Run)
            .await?;

        let (context, rx) = FuncDispatchContext::new(ctx);
        Ok((func, execution, context, rx))
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LogLinePayload {
    pub line: OutputStream,
    pub func_id: FuncId,
    pub execution_key: String,
}

impl WsEvent {
    pub async fn log_line(ctx: &DalContext, payload: LogLinePayload) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::LogLine(payload)).await
    }
}
