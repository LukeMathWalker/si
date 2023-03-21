use std::{collections::HashMap, collections::HashSet, convert::TryFrom, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use tokio::{sync::Mutex, task::JoinSet};

use crate::tasks::StatusReceiverClient;
use crate::tasks::StatusReceiverRequest;
use crate::{
    job::consumer::{
        JobConsumer, JobConsumerError, JobConsumerMetadata, JobConsumerResult, JobInfo,
    },
    job::producer::{JobMeta, JobProducer, JobProducerResult},
    AccessBuilder, AttributeValue, AttributeValueError, AttributeValueId, AttributeValueResult,
    DalContext, StandardModel, StatusUpdater, Visibility, WsEvent,
};

#[derive(Debug, Deserialize, Serialize)]
struct DependentValuesUpdateArgs {
    attribute_values: Vec<AttributeValueId>,
}

impl From<DependentValuesUpdate> for DependentValuesUpdateArgs {
    fn from(value: DependentValuesUpdate) -> Self {
        Self {
            attribute_values: value.attribute_values,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DependentValuesUpdate {
    attribute_values: Vec<AttributeValueId>,
    access_builder: AccessBuilder,
    visibility: Visibility,
    job: Option<JobInfo>,
}

impl DependentValuesUpdate {
    pub fn new(ctx: &DalContext, attribute_values: Vec<AttributeValueId>) -> Box<Self> {
        let access_builder = AccessBuilder::from(ctx.clone());
        let visibility = *ctx.visibility();

        Box::new(Self {
            attribute_values,
            access_builder,
            visibility,
            job: None,
        })
    }
}

impl JobProducer for DependentValuesUpdate {
    fn args(&self) -> JobProducerResult<serde_json::Value> {
        Ok(serde_json::to_value(DependentValuesUpdateArgs::from(
            self.clone(),
        ))?)
    }

    fn meta(&self) -> JobProducerResult<JobMeta> {
        let mut custom = HashMap::new();
        custom.insert(
            "access_builder".to_string(),
            serde_json::to_value(self.access_builder.clone())?,
        );
        custom.insert(
            "visibility".to_string(),
            serde_json::to_value(self.visibility)?,
        );

        Ok(JobMeta {
            retry: Some(0),
            custom,
            ..JobMeta::default()
        })
    }

    fn identity(&self) -> String {
        serde_json::to_string(self).expect("Cannot serialize DependentValueUpdate")
    }
}

impl JobConsumerMetadata for DependentValuesUpdate {
    fn type_name(&self) -> String {
        "DependentValuesUpdate".to_string()
    }

    fn access_builder(&self) -> AccessBuilder {
        self.access_builder.clone()
    }

    fn visibility(&self) -> Visibility {
        self.visibility
    }
}

#[async_trait]
impl JobConsumer for DependentValuesUpdate {
    async fn run(&self, ctx: &DalContext) -> JobConsumerResult<()> {
        let mut ctx = ctx.clone_with_new_transactions().await?;

        let now = std::time::Instant::now();

        let status_updater = Arc::new(Mutex::new(StatusUpdater::initialize(&ctx).await?));

        let jid = council_server::Id::from_string(&self.job.as_ref().unwrap().id)?;
        let mut council = council_server::Client::new(
            ctx.nats_conn().clone(),
            ctx.council_subject_prefix(),
            jid,
            self.visibility().change_set_pk.into(),
        )
        .await?;
        let pub_council = council.clone_into_pub();

        if let council_server::client::State::Shutdown = council.wait_to_create_values().await? {
            return Ok(());
        }

        AttributeValue::create_dependent_values(&ctx, &self.attribute_values).await?;

        ctx = ctx.commit_and_continue().await?;

        council.finished_creating_values().await?;

        let mut dependency_graph =
            AttributeValue::dependent_value_graph(&ctx, &self.attribute_values).await?;

        // NOTE(nick,jacob): uncomment this for debugging.
        // Save printed output to a file and execute the following: "dot <file> -Tsvg -o <newfile>.svg"
        // println!("{}", dependency_graph_to_dot(ctx, &dependency_graph).await?);

        // Remove the `AttributeValueIds` from the list of values that are in the dependencies,
        // as we consider that one to have already been updated. This lets us check for
        // `AttributeValuesId`s where the list of *unsatisfied* dependencies is empty.
        let attribute_values_set: HashSet<AttributeValueId> =
            HashSet::from_iter(self.attribute_values.iter().cloned());
        let mut to_remove = Vec::new();
        for (id, val) in dependency_graph.iter_mut() {
            val.retain(|id| !attribute_values_set.contains(id));
            if val.is_empty() {
                to_remove.push(*id);
            }
        }

        for id in to_remove {
            dependency_graph.remove(&id);
        }

        info!(
            "DependentValuesUpdate for {:?}: dependency_graph {:?}",
            self.attribute_values, &dependency_graph
        );

        if dependency_graph.is_empty() {
            return Ok(());
        }

        // Cache the original dependency graph to send the status receiver.
        let original_dependency_graph = dependency_graph.clone();

        council
            .register_dependency_graph(
                dependency_graph
                    .iter()
                    .map(|(key, value)| (key.into(), value.iter().map(Into::into).collect()))
                    .collect(),
            )
            .await?;

        let mut enqueued: Vec<AttributeValueId> = dependency_graph.keys().copied().collect();
        enqueued.extend(dependency_graph.values().flatten().copied());
        status_updater
            .lock()
            .await
            .values_queued(&ctx, enqueued)
            .await?;

        ctx = ctx.commit_and_continue().await?;

        let mut update_tasks = JoinSet::new();

        while !dependency_graph.is_empty() {
            match council.fetch_response().await? {
                Some(response) => match response {
                    council_server::Response::OkToProcess { node_ids } => {
                        for node_id in node_ids {
                            let id = AttributeValueId::from(node_id);
                            dependency_graph.remove(&id);

                            status_updater
                                .lock()
                                .await
                                .values_running(&ctx, vec![id])
                                .await?;
                            ctx = ctx.commit_and_continue().await?;

                            let task_ctx = ctx.clone_with_new_transactions().await?;
                            let attribute_value = AttributeValue::get_by_id(&task_ctx, &id)
                                .await?
                                .ok_or_else(|| {
                                    AttributeValueError::NotFound(id, self.visibility())
                                })?;
                            update_tasks
                                .build_task()
                                .name("AttributeValue.update_from_prototype_function")
                                .spawn(update_value(
                                    task_ctx,
                                    attribute_value,
                                    pub_council.clone(),
                                    status_updater.clone(),
                                ))?;
                        }
                    }
                    council_server::Response::BeenProcessed { node_id } => {
                        let id = AttributeValueId::from(node_id);
                        dependency_graph.remove(&id);

                        // Send a completed status for this value and *remove* it from the hash
                        status_updater
                            .lock()
                            .await
                            .values_running(&ctx, vec![id])
                            .await?;
                        status_updater
                            .lock()
                            .await
                            .values_completed(&ctx, vec![id])
                            .await?;

                        ctx = ctx.commit_and_continue().await?;
                    }
                    council_server::Response::OkToCreate => unreachable!(),
                    council_server::Response::Shutdown => break,
                },
                // FIXME: reconnect
                None => break, // Happens if subscription has been unsubscribed or if connection is closed
            }

            WsEvent::change_set_written(&ctx)
                .await?
                .publish_on_commit(&ctx)
                .await?;
            ctx = ctx.commit_and_continue().await?;

            // If we get `None` back from the `JoinSet` that means that there are no
            // further tasks in the `JoinSet` for us to wait on. This should only happen
            // after we've stopped adding new tasks to the `JoinSet`, which means either:
            //   * We have completely walked the initial graph, and have visited every
            //     node.
            //   * We've encountered a cycle that means we can no longer make any
            //     progress on walking the graph.
            // In both cases, there isn't anything more we can do, so we can stop looking
            // at the graph to find more work.
            while let Some(future_result) = update_tasks.join_next().await {
                // We get back a `Some<Result<Result<..>>>`. We've already unwrapped the
                // `Some`, the outermost `Result` is a `JoinError` to let us know if
                // anything went wrong in joining the task.
                match future_result {
                    // We have successfully updated a value
                    Ok(Ok(())) => {}
                    // There was an error (with our code) when updating the value
                    Ok(Err(err)) => {
                        warn!(error = ?err, "error updating value");

                        council.bye().await?;
                        return Err(err.into());
                    }
                    // There was a Tokio JoinSet error when joining the task back (i.e. likely
                    // I/O error)
                    Err(err) => {
                        warn!(error = ?err, "error when joining update task");

                        council.bye().await?;
                        return Err(err.into());
                    }
                }
            }
        }

        ctx = ctx.commit_and_continue().await?;

        Arc::try_unwrap(status_updater)
            .unwrap()
            .into_inner()
            .finish(&ctx)
            .await?;

        WsEvent::change_set_written(&ctx)
            .await?
            .publish_on_commit(&ctx)
            .await?;

        let client = StatusReceiverClient::new(ctx.nats_conn().clone()).await;
        if let Err(e) = client
            .publish(&StatusReceiverRequest {
                visibility: *ctx.visibility(),
                tenancy: *ctx.tenancy(),
                dependent_graph: original_dependency_graph,
            })
            .await
        {
            error!("could not publish status receiver request: {:?}", e);
        }

        ctx.commit().await?;

        let elapsed_time = now.elapsed();
        info!(
            "DependentValuesUpdate for {:?} took {:?}",
            &self.attribute_values, elapsed_time
        );

        council.bye().await?;

        Ok(())
    }
}

/// Wrapper around `AttributeValue.update_from_prototype_function(&ctx)` to get it to
/// play more nicely with being spawned into a `JoinSet`.
async fn update_value(
    ctx: DalContext,
    mut attribute_value: AttributeValue,
    council: council_server::PubClient,
    status_updater: Arc<Mutex<StatusUpdater>>,
) -> AttributeValueResult<()> {
    info!("DependentValueUpdate {:?}: START", attribute_value.id());
    let start = std::time::Instant::now();
    attribute_value.update_from_prototype_function(&ctx).await?;
    info!(
        "DependentValueUpdate {:?}: DONE {:?}",
        attribute_value.id(),
        start.elapsed()
    );

    // Send a completed status for this value and *remove* it from the hash
    status_updater
        .lock()
        .await
        .values_completed(&ctx, vec![*attribute_value.id()])
        .await
        .map_err(Box::new)?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;
    ctx.commit().await?;

    council.processed_value(attribute_value.id().into()).await?;

    Ok(())
}

impl TryFrom<JobInfo> for DependentValuesUpdate {
    type Error = JobConsumerError;

    fn try_from(job: JobInfo) -> Result<Self, Self::Error> {
        if job.args().len() != 3 {
            return Err(JobConsumerError::InvalidArguments(
                r#"[{ "attribute_values": <Vec<AttributeValueId>> }, <AccessBuilder>, <Visibility>]"#
                    .to_string(),
                job.args().to_vec(),
            ));
        }
        let args: DependentValuesUpdateArgs = serde_json::from_value(job.args()[0].clone())?;
        let access_builder: AccessBuilder = serde_json::from_value(job.args()[1].clone())?;
        let visibility: Visibility = serde_json::from_value(job.args()[2].clone())?;

        Ok(Self {
            attribute_values: args.attribute_values,
            access_builder,
            visibility,
            job: Some(job),
        })
    }
}

#[allow(unused)]
async fn dependency_graph_to_dot(
    ctx: &DalContext,
    graph: &HashMap<AttributeValueId, Vec<AttributeValueId>>,
) -> AttributeValueResult<String> {
    let mut node_definitions = String::new();
    for attr_val_id in graph.keys() {
        let attr_val = AttributeValue::get_by_id(ctx, attr_val_id)
            .await?
            .ok_or_else(|| AttributeValueError::NotFound(*attr_val_id, *ctx.visibility()))?;
        let prop_id = attr_val.context.prop_id();
        let internal_provider_id = attr_val.context.internal_provider_id();
        let external_provider_id = attr_val.context.external_provider_id();
        let component_id = attr_val.context.component_id();
        node_definitions.push_str(&format!(
            "\"{attr_val_id}\"[label=\"\\lAttribute Value: {attr_val_id}\\n\\lProp: {prop_id}\\lInternal Provider: {internal_provider_id}\\lExternal Provider: {external_provider_id}\\lComponent: {component_id}\"];",
        ));
    }

    let mut node_graph = String::new();
    for (attr_val, inputs) in graph {
        let dependencies = format!(
            "{{{dep_list}}}",
            dep_list = inputs
                .iter()
                .map(|i| format!("\"{i}\""))
                .collect::<Vec<String>>()
                .join(" ")
        );
        let dependency_line = format!("{dependencies} -> \"{attr_val}\";",);
        node_graph.push_str(&dependency_line);
    }

    let dot_digraph = format!("digraph G {{{node_definitions}{node_graph}}}");

    Ok(dot_digraph)
}
