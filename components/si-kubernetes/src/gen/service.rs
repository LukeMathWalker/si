// Auto-generated rust code!
// No-Touchy!

use opentelemetry::api::propagation::text_propagator::HttpTextFormat;
use tracing_futures::Instrument as _;
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

pub use crate::protobuf::kubernetes_server::KubernetesServer as Server;

#[derive(Debug)]
pub struct Service {
    db: si_data::Db,
    agent: si_cea::AgentClient,
}

impl Service {
    pub fn new(db: si_data::Db, agent: si_cea::AgentClient) -> Service {
        Service { db, agent }
    }

    pub async fn migrate(&self) -> si_data::Result<()> {
        crate::protobuf::KubernetesDeploymentComponent::migrate(&self.db).await?;
        crate::protobuf::KubernetesServiceComponent::migrate(&self.db).await?;

        Ok(())
    }
}

#[tonic::async_trait]
impl crate::protobuf::kubernetes_server::Kubernetes for Service {
    async fn kubernetes_deployment_component_create(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesDeploymentComponentCreateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentComponentCreateReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_deployment_component_create",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_deployment_component_create",
            )
            .await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let description = inner.description;
            let constraints = inner.constraints;
            let si_properties = inner.si_properties;

            let output = crate::protobuf::KubernetesDeploymentComponent::create(
                &self.db,
                name,
                display_name,
                description,
                constraints,
                si_properties,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesDeploymentComponentCreateReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_deployment_component_get(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesDeploymentComponentGetRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentComponentGetReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_deployment_component_get",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_deployment_component_get",
            )
            .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::KubernetesDeploymentComponent::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesDeploymentComponentGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_deployment_component_list(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesDeploymentComponentListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentComponentListReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_deployment_component_list",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            #[allow(unused_variables)]
            let auth = si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_deployment_component_list",
            )
            .await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(String::from("global"));
            }

            let output =
                crate::protobuf::KubernetesDeploymentComponent::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesDeploymentComponentListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_deployment_component_pick(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesDeploymentComponentPickRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentComponentPickReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_deployment_component_pick",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_deployment_component_pick",
            )
            .await?;

            let inner = request.into_inner();
            let constraints = inner.constraints;

            let (implicit_constraints, component) =
                crate::protobuf::KubernetesDeploymentComponent::pick(&self.db, constraints).await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesDeploymentComponentPickReply {
                    implicit_constraints: Some(implicit_constraints),
                    component: Some(component),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_deployment_entity_apply(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesDeploymentEntityApplyRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentEntityApplyReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_deployment_entity_apply",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            use si_cea::EntityEvent;

            let auth = si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_deployment_entity_apply",
            )
            .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let entity = crate::protobuf::KubernetesDeploymentEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::KubernetesDeploymentEntityEvent::create(
                &self.db,
                auth.user_id(),
                "apply",
                &entity,
            )
            .await?;

            // You belong to execute now, homie
            //self.agent.dispatch(&entity_event).await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesDeploymentEntityApplyReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_deployment_entity_create(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesDeploymentEntityCreateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentEntityCreateReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_deployment_entity_create",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            use si_cea::EntityEvent;

            let auth = si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_deployment_entity_create",
            )
            .await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let description = inner.description;
            let workspace_id = inner.workspace_id;
            let change_set_id = inner.change_set_id;
            let properties = inner.properties;
            let constraints = inner.constraints;

            let workspace_id = workspace_id.ok_or_else(|| {
                si_data::DataError::ValidationError(
                    "missing required workspace_id value".to_string(),
                )
            })?;

            let workspace = si_account::Workspace::get(&self.db, &workspace_id).await?;

            let (implicit_constraints, component) =
                crate::protobuf::KubernetesDeploymentComponent::pick(&self.db, constraints.clone())
                    .await?;

            let si_properties = si_cea::EntitySiProperties::new(
                &workspace,
                component
                    .id
                    .as_ref()
                    .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?,
                component.si_properties.as_ref().ok_or_else(|| {
                    si_data::DataError::RequiredField("si_properties".to_string())
                })?,
            )?;

            let entity = crate::protobuf::KubernetesDeploymentEntity::create(
                &self.db,
                name,
                display_name,
                description,
                constraints,
                Some(implicit_constraints),
                properties,
                Some(si_properties),
                change_set_id,
            )
            .await?;
            //let entity_event = crate::protobuf::KubernetesDeploymentEntityEvent::create(
            //    &self.db,
            //    auth.user_id(),
            //    "create",
            //    &entity,
            //).await?;
            //self.agent.dispatch(&entity_event).await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesDeploymentEntityCreateReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_deployment_entity_delete(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesDeploymentEntityDeleteRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentEntityDeleteReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_deployment_entity_delete",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_deployment_entity_delete",
            )
            .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity =
                crate::protobuf::KubernetesDeploymentEntity::get(&self.db, &id).await?;

            entity.delete(&self.db, inner.change_set_id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesDeploymentEntityDeleteReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_deployment_entity_get(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesDeploymentEntityGetRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentEntityGetReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_deployment_entity_get",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            si_account::authorize::authnz(&self.db, &request, "kubernetes_deployment_entity_get")
                .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::KubernetesDeploymentEntity::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesDeploymentEntityGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_deployment_entity_kubernetes_object_edit(
        &self,
        mut request: tonic::Request<
            crate::protobuf::KubernetesDeploymentEntityKubernetesObjectEditRequest,
        >,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentEntityKubernetesObjectEditReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_deployment_entity_kubernetes_object_edit",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            use si_cea::{Entity, EntityEvent};

            let auth = si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_deployment_entity_kubernetes_object_edit",
            )
            .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let property = inner
                .property
                .ok_or_else(|| si_data::DataError::RequiredField("property".to_string()))?;

            let mut entity =
                crate::protobuf::KubernetesDeploymentEntity::get(&self.db, &id).await?;
            let previous_entity = entity.clone();

            entity.set_entity_state_transition();
            entity.edit_kubernetes_object(property)?;
            entity.save(&self.db).await?;

            let entity_event =
                crate::protobuf::KubernetesDeploymentEntityEvent::create_with_previous_entity(
                    &self.db,
                    auth.user_id(),
                    "edit_kubernetes_object",
                    &entity,
                    previous_entity,
                )
                .await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesDeploymentEntityKubernetesObjectEditReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_deployment_entity_kubernetes_object_yaml_edit(
        &self,
        mut request: tonic::Request<
            crate::protobuf::KubernetesDeploymentEntityKubernetesObjectYamlEditRequest,
        >,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentEntityKubernetesObjectYamlEditReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_deployment_entity_kubernetes_object_yaml_edit",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            use si_cea::{Entity, EntityEvent};

            let auth = si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_deployment_entity_kubernetes_object_yaml_edit",
            )
            .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let property = inner
                .property
                .ok_or_else(|| si_data::DataError::RequiredField("property".to_string()))?;

            let mut entity =
                crate::protobuf::KubernetesDeploymentEntity::get(&self.db, &id).await?;
            let previous_entity = entity.clone();

            entity.set_entity_state_transition();
            entity.edit_kubernetes_object_yaml(property)?;
            entity.save(&self.db).await?;

            let entity_event =
                crate::protobuf::KubernetesDeploymentEntityEvent::create_with_previous_entity(
                    &self.db,
                    auth.user_id(),
                    "edit_kubernetes_object_yaml",
                    &entity,
                    previous_entity,
                )
                .await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesDeploymentEntityKubernetesObjectYamlEditReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_deployment_entity_list(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesDeploymentEntityListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentEntityListReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_deployment_entity_list",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            #[allow(unused_variables)]
            let auth = si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_deployment_entity_list",
            )
            .await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::KubernetesDeploymentEntity::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesDeploymentEntityListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_deployment_entity_sync(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesDeploymentEntitySyncRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentEntitySyncReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_deployment_entity_sync",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            use si_cea::EntityEvent;

            let auth = si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_deployment_entity_sync",
            )
            .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let entity = crate::protobuf::KubernetesDeploymentEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::KubernetesDeploymentEntityEvent::create(
                &self.db,
                auth.user_id(),
                "sync",
                &entity,
            )
            .await?;

            // You belong to execute now, homie
            //self.agent.dispatch(&entity_event).await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesDeploymentEntitySyncReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_deployment_entity_update(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesDeploymentEntityUpdateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentEntityUpdateReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_deployment_entity_update",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_deployment_entity_update",
            )
            .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity =
                crate::protobuf::KubernetesDeploymentEntity::get(&self.db, &id).await?;

            entity
                .update(&self.db, inner.change_set_id, inner.update)
                .await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesDeploymentEntityUpdateReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_deployment_entity_event_list(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesDeploymentEntityEventListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentEntityEventListReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_deployment_entity_event_list",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            #[allow(unused_variables)]
            let auth = si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_deployment_entity_event_list",
            )
            .await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output =
                crate::protobuf::KubernetesDeploymentEntityEvent::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesDeploymentEntityEventListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_service_component_create(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesServiceComponentCreateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesServiceComponentCreateReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_service_component_create",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_service_component_create",
            )
            .await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let description = inner.description;
            let constraints = inner.constraints;
            let si_properties = inner.si_properties;

            let output = crate::protobuf::KubernetesServiceComponent::create(
                &self.db,
                name,
                display_name,
                description,
                constraints,
                si_properties,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesServiceComponentCreateReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_service_component_get(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesServiceComponentGetRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesServiceComponentGetReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_service_component_get",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            si_account::authorize::authnz(&self.db, &request, "kubernetes_service_component_get")
                .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::KubernetesServiceComponent::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesServiceComponentGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_service_component_list(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesServiceComponentListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesServiceComponentListReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_service_component_list",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            #[allow(unused_variables)]
            let auth = si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_service_component_list",
            )
            .await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(String::from("global"));
            }

            let output = crate::protobuf::KubernetesServiceComponent::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesServiceComponentListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_service_component_pick(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesServiceComponentPickRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesServiceComponentPickReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_service_component_pick",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            si_account::authorize::authnz(&self.db, &request, "kubernetes_service_component_pick")
                .await?;

            let inner = request.into_inner();
            let constraints = inner.constraints;

            let (implicit_constraints, component) =
                crate::protobuf::KubernetesServiceComponent::pick(&self.db, constraints).await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesServiceComponentPickReply {
                    implicit_constraints: Some(implicit_constraints),
                    component: Some(component),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_service_entity_create(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesServiceEntityCreateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesServiceEntityCreateReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_service_entity_create",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            use si_cea::EntityEvent;

            let auth = si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_service_entity_create",
            )
            .await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let description = inner.description;
            let workspace_id = inner.workspace_id;
            let change_set_id = inner.change_set_id;
            let properties = inner.properties;
            let constraints = inner.constraints;

            let workspace_id = workspace_id.ok_or_else(|| {
                si_data::DataError::ValidationError(
                    "missing required workspace_id value".to_string(),
                )
            })?;

            let workspace = si_account::Workspace::get(&self.db, &workspace_id).await?;

            let (implicit_constraints, component) =
                crate::protobuf::KubernetesServiceComponent::pick(&self.db, constraints.clone())
                    .await?;

            let si_properties = si_cea::EntitySiProperties::new(
                &workspace,
                component
                    .id
                    .as_ref()
                    .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?,
                component.si_properties.as_ref().ok_or_else(|| {
                    si_data::DataError::RequiredField("si_properties".to_string())
                })?,
            )?;

            let entity = crate::protobuf::KubernetesServiceEntity::create(
                &self.db,
                name,
                display_name,
                description,
                constraints,
                Some(implicit_constraints),
                properties,
                Some(si_properties),
                change_set_id,
            )
            .await?;
            //let entity_event = crate::protobuf::KubernetesServiceEntityEvent::create(
            //    &self.db,
            //    auth.user_id(),
            //    "create",
            //    &entity,
            //).await?;
            //self.agent.dispatch(&entity_event).await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesServiceEntityCreateReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_service_entity_delete(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesServiceEntityDeleteRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesServiceEntityDeleteReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_service_entity_delete",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            si_account::authorize::authnz(&self.db, &request, "kubernetes_service_entity_delete")
                .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity = crate::protobuf::KubernetesServiceEntity::get(&self.db, &id).await?;

            entity.delete(&self.db, inner.change_set_id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesServiceEntityDeleteReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_service_entity_get(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesServiceEntityGetRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesServiceEntityGetReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_service_entity_get",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            si_account::authorize::authnz(&self.db, &request, "kubernetes_service_entity_get")
                .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::KubernetesServiceEntity::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesServiceEntityGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_service_entity_kubernetes_object_edit(
        &self,
        mut request: tonic::Request<
            crate::protobuf::KubernetesServiceEntityKubernetesObjectEditRequest,
        >,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesServiceEntityKubernetesObjectEditReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_service_entity_kubernetes_object_edit",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            use si_cea::{Entity, EntityEvent};

            let auth = si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_service_entity_kubernetes_object_edit",
            )
            .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let property = inner
                .property
                .ok_or_else(|| si_data::DataError::RequiredField("property".to_string()))?;

            let mut entity = crate::protobuf::KubernetesServiceEntity::get(&self.db, &id).await?;
            let previous_entity = entity.clone();

            entity.set_entity_state_transition();
            entity.edit_kubernetes_object(property)?;
            entity.save(&self.db).await?;

            let entity_event =
                crate::protobuf::KubernetesServiceEntityEvent::create_with_previous_entity(
                    &self.db,
                    auth.user_id(),
                    "edit_kubernetes_object",
                    &entity,
                    previous_entity,
                )
                .await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesServiceEntityKubernetesObjectEditReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_service_entity_kubernetes_object_yaml_edit(
        &self,
        mut request: tonic::Request<
            crate::protobuf::KubernetesServiceEntityKubernetesObjectYamlEditRequest,
        >,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesServiceEntityKubernetesObjectYamlEditReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_service_entity_kubernetes_object_yaml_edit",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            use si_cea::{Entity, EntityEvent};

            let auth = si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_service_entity_kubernetes_object_yaml_edit",
            )
            .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let property = inner
                .property
                .ok_or_else(|| si_data::DataError::RequiredField("property".to_string()))?;

            let mut entity = crate::protobuf::KubernetesServiceEntity::get(&self.db, &id).await?;
            let previous_entity = entity.clone();

            entity.set_entity_state_transition();
            entity.edit_kubernetes_object_yaml(property)?;
            entity.save(&self.db).await?;

            let entity_event =
                crate::protobuf::KubernetesServiceEntityEvent::create_with_previous_entity(
                    &self.db,
                    auth.user_id(),
                    "edit_kubernetes_object_yaml",
                    &entity,
                    previous_entity,
                )
                .await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesServiceEntityKubernetesObjectYamlEditReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_service_entity_list(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesServiceEntityListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesServiceEntityListReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_service_entity_list",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            #[allow(unused_variables)]
            let auth =
                si_account::authorize::authnz(&self.db, &request, "kubernetes_service_entity_list")
                    .await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::KubernetesServiceEntity::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesServiceEntityListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_service_entity_sync(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesServiceEntitySyncRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesServiceEntitySyncReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_service_entity_sync",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            use si_cea::EntityEvent;

            let auth =
                si_account::authorize::authnz(&self.db, &request, "kubernetes_service_entity_sync")
                    .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let entity = crate::protobuf::KubernetesServiceEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::KubernetesServiceEntityEvent::create(
                &self.db,
                auth.user_id(),
                "sync",
                &entity,
            )
            .await?;

            // You belong to execute now, homie
            //self.agent.dispatch(&entity_event).await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesServiceEntitySyncReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_service_entity_update(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesServiceEntityUpdateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesServiceEntityUpdateReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_service_entity_update",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            si_account::authorize::authnz(&self.db, &request, "kubernetes_service_entity_update")
                .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity = crate::protobuf::KubernetesServiceEntity::get(&self.db, &id).await?;

            entity
                .update(&self.db, inner.change_set_id, inner.update)
                .await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesServiceEntityUpdateReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn kubernetes_service_entity_event_list(
        &self,
        mut request: tonic::Request<crate::protobuf::KubernetesServiceEntityEventListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesServiceEntityEventListReply>,
        tonic::Status,
    > {
        let trace_propagator =
            opentelemetry::api::trace::trace_context_propagator::TraceContextPropagator::new();
        let span_context = {
            let metadata_wrapper = TonicMetaWrapper(request.metadata_mut());
            trace_propagator.extract(&metadata_wrapper)
        };
        let span = tracing::span!(
            tracing::Level::INFO,
            "kubernetes.kubernetes_service_entity_event_list",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            #[allow(unused_variables)]
            let auth = si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_service_entity_event_list",
            )
            .await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output =
                crate::protobuf::KubernetesServiceEntityEvent::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::KubernetesServiceEntityEventListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }
}

struct TonicMetaWrapper<'a>(&'a mut tonic::metadata::MetadataMap);

impl<'a> opentelemetry::api::propagation::Carrier for TonicMetaWrapper<'a> {
    fn get(&self, key: &'static str) -> Option<&str> {
        let raw_value = self.0.get(key)?;
        match raw_value.to_str() {
            Ok(value) => Some(value),
            Err(_e) => {
                tracing::debug!("Cannot extract header for trace parent, not a string");
                None
            }
        }
    }

    fn set(&mut self, key: &'static str, raw_value: String) {
        let value = match tonic::metadata::MetadataValue::from_str(&raw_value) {
            Ok(value) => value,
            Err(_e) => {
                tracing::debug!("Cannot insert header for trace parent, not a string");
                tracing::debug!("Inserting the empty string");
                tonic::metadata::MetadataValue::from_str("").unwrap()
            }
        };
        self.0.insert(key, value);
    }
}
