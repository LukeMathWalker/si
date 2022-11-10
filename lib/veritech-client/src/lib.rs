use std::sync::Arc;

use futures::{StreamExt, TryStreamExt};
use serde::{de::DeserializeOwned, Serialize};
use si_data_nats::NatsClient;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;
use veritech_core::{
    nats_code_generation_subject, nats_command_run_subject, nats_confirmation_subject,
    nats_qualification_check_subject, nats_resolver_function_subject, nats_subject,
    nats_validation_subject, nats_workflow_resolve_subject, reply_mailbox_for_output,
    reply_mailbox_for_result,
};

mod subscription;
use subscription::{Subscription, SubscriptionError};

pub use cyclone_core::{
    CodeGenerated, CodeGenerationRequest, CodeGenerationResultSuccess, CommandRunRequest,
    CommandRunResultSuccess, ComponentKind, ComponentView, ConfirmationRequest,
    ConfirmationResultSuccess, EncryptionKey, EncryptionKeyError, FunctionResult,
    FunctionResultFailure, OutputStream, QualificationCheckComponent, QualificationCheckRequest,
    QualificationCheckResultSuccess, QualificationSubCheck, QualificationSubCheckStatus,
    ResolverFunctionComponent, ResolverFunctionRequest, ResolverFunctionResultSuccess,
    ResourceView, SensitiveContainer, SystemView, ValidationRequest, ValidationResultSuccess,
    WorkflowResolveRequest, WorkflowResolveResultSuccess,
};

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("failed to serialize json message")]
    JSONSerialize(#[source] serde_json::Error),
    #[error("nats error")]
    Nats(#[from] si_data_nats::NatsError),
    #[error("no function result from cyclone; bug!")]
    NoResult,
    #[error("result error")]
    Result(#[from] SubscriptionError),
}

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Clone, Debug)]
pub struct Client {
    nats: NatsClient,
    subject_prefix: Option<Arc<String>>,
}

impl Client {
    pub fn new(nats: NatsClient) -> Self {
        Self {
            nats,
            subject_prefix: None,
        }
    }

    pub fn with_subject_prefix(nats: NatsClient, subject_prefix: impl Into<String>) -> Self {
        Self {
            nats,
            subject_prefix: Some(Arc::new(subject_prefix.into())),
        }
    }

    #[instrument(name = "client.execute_qualification_check", skip_all)]
    pub async fn execute_qualification_check(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &QualificationCheckRequest,
    ) -> ClientResult<FunctionResult<QualificationCheckResultSuccess>> {
        self.execute_request(
            nats_qualification_check_subject(self.subject_prefix()),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_qualification_check_with_subject", skip_all)]
    pub async fn execute_qualification_check_with_subject(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &QualificationCheckRequest,
        subject_suffix: impl AsRef<str>,
    ) -> ClientResult<FunctionResult<QualificationCheckResultSuccess>> {
        self.execute_request(
            nats_subject(self.subject_prefix(), subject_suffix),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_confirmation", skip_all)]
    pub async fn execute_confirmation(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ConfirmationRequest,
    ) -> ClientResult<FunctionResult<ConfirmationResultSuccess>> {
        self.execute_request(
            nats_confirmation_subject(self.subject_prefix()),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_confirmation_with_subject", skip_all)]
    pub async fn execute_confirmation_with_subject(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ConfirmationRequest,
        subject_suffix: impl AsRef<str>,
    ) -> ClientResult<FunctionResult<ConfirmationResultSuccess>> {
        self.execute_request(
            nats_subject(self.subject_prefix(), subject_suffix),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_resolver_function", skip_all)]
    pub async fn execute_resolver_function(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ResolverFunctionRequest,
    ) -> ClientResult<FunctionResult<ResolverFunctionResultSuccess>> {
        self.execute_request(
            nats_resolver_function_subject(self.subject_prefix()),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_resolver_function_with_subject", skip_all)]
    pub async fn execute_resolver_function_with_subject(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ResolverFunctionRequest,
        subject_suffix: impl AsRef<str>,
    ) -> ClientResult<FunctionResult<ResolverFunctionResultSuccess>> {
        self.execute_request(
            nats_subject(self.subject_prefix(), subject_suffix),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_code_generation", skip_all)]
    pub async fn execute_code_generation(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &CodeGenerationRequest,
    ) -> ClientResult<FunctionResult<CodeGenerationResultSuccess>> {
        self.execute_request(
            nats_code_generation_subject(self.subject_prefix()),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_code_generation_with_subject", skip_all)]
    pub async fn execute_code_generation_with_subject(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &CodeGenerationRequest,
        subject_suffix: impl AsRef<str>,
    ) -> ClientResult<FunctionResult<CodeGenerationResultSuccess>> {
        self.execute_request(
            nats_subject(self.subject_prefix(), subject_suffix),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_validation", skip_all)]
    pub async fn execute_validation(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ValidationRequest,
    ) -> ClientResult<FunctionResult<ValidationResultSuccess>> {
        self.execute_request(
            nats_validation_subject(self.subject_prefix()),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_validation_with_subject", skip_all)]
    pub async fn execute_validation_with_subject(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ValidationResultSuccess,
        subject_suffix: impl AsRef<str>,
    ) -> ClientResult<FunctionResult<ValidationResultSuccess>> {
        self.execute_request(
            nats_subject(self.subject_prefix(), subject_suffix),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_workflow_resolve", skip_all)]
    pub async fn execute_workflow_resolve(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &WorkflowResolveRequest,
    ) -> ClientResult<FunctionResult<WorkflowResolveResultSuccess>> {
        self.execute_request(
            nats_workflow_resolve_subject(self.subject_prefix()),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_workflow_resolve_with_subject", skip_all)]
    pub async fn execute_workflow_resolve_with_subject(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &WorkflowResolveRequest,
        subject_suffix: impl AsRef<str>,
    ) -> ClientResult<FunctionResult<WorkflowResolveResultSuccess>> {
        self.execute_request(
            nats_subject(self.subject_prefix(), subject_suffix),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_command_run", skip_all)]
    pub async fn execute_command_run(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &CommandRunRequest,
    ) -> ClientResult<FunctionResult<CommandRunResultSuccess>> {
        self.execute_request(
            nats_command_run_subject(self.subject_prefix()),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_command_run_with_subject", skip_all)]
    pub async fn execute_command_run_with_subject(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &CommandRunRequest,
        subject_suffix: impl AsRef<str>,
    ) -> ClientResult<FunctionResult<CommandRunResultSuccess>> {
        self.execute_request(
            nats_subject(self.subject_prefix(), subject_suffix),
            output_tx,
            request,
        )
        .await
    }

    async fn execute_request<R, S>(
        &self,
        subject: impl Into<String>,
        output_tx: mpsc::Sender<OutputStream>,
        request: &R,
    ) -> ClientResult<FunctionResult<S>>
    where
        R: Serialize,
        S: DeserializeOwned,
    {
        let msg = serde_json::to_vec(request).map_err(ClientError::JSONSerialize)?;
        let reply_mailbox_root = self.nats.new_inbox();

        // Construct a subscription stream for the result
        let result_subscription_subject = reply_mailbox_for_result(&reply_mailbox_root);
        error!(
            messaging.destination = &result_subscription_subject.as_str(),
            "subscribing for result messages"
        );
        let mut result_subscription: Subscription<FunctionResult<S>> =
            Subscription::new(self.nats.subscribe(result_subscription_subject.clone()).await?);

        // Construct a subscription stream for output messages
        let output_subscription_subject = reply_mailbox_for_output(&reply_mailbox_root);
        error!(
            messaging.destination = &output_subscription_subject.as_str(),
            "subscribing for output messages"
        );
        let output_subscription =
            Subscription::new(self.nats.subscribe(output_subscription_subject).await?);
        // Spawn a task to forward output to the sender provided by the caller
        tokio::spawn(forward_output_task(output_subscription, output_tx));

        // Submit the request message
        let subject = subject.into();
        error!(
            messaging.destination = &subject.as_str(),
            "publishing message"
        );
        self.nats
            .publish_with_reply_or_headers(subject, Some(reply_mailbox_root.as_str()), None, msg)
            .await?;

        error!(
            messaging.destination = &result_subscription_subject,
            "waiting for result message"
        );
        // Wait for one message on the result reply mailbox
        let result = result_subscription
            .try_next()
            .await?
            .ok_or(ClientError::NoResult)?;
        result_subscription.unsubscribe().await?;

        error!("got it");
        Ok(result)
    }

    /// Gets a reference to the client's subject prefix.
    pub fn subject_prefix(&self) -> Option<&str> {
        self.subject_prefix.as_deref().map(String::as_str)
    }
}

async fn forward_output_task(
    mut output_subscription: Subscription<OutputStream>,
    output_tx: mpsc::Sender<OutputStream>,
) {
    while let Some(msg) = output_subscription.next().await {
        match msg {
            Ok(output) => {
                if let Err(err) = output_tx.send(output).await {
                    warn!(error = ?err, "output forwarder failed to send message on channel");
                }
            }
            Err(err) => {
                warn!(error = ?err, "output forwarder received an error on its subscription")
            }
        }
    }
    if let Err(err) = output_subscription.unsubscribe().await {
        warn!(error = ?err, "error when unsubscribing from output subscription");
    }
}

#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use std::env;

    use indoc::indoc;
    use si_data_nats::NatsConfig;
    use test_log::test;
    use tokio::task::JoinHandle;
    use uuid::Uuid;
    use veritech_server::{
        Config, CycloneSpec, Instance, LocalUdsInstance, Server, ServerError, StandardConfig,
    };

    use super::*;

    fn nats_config() -> NatsConfig {
        let mut config = NatsConfig::default();
        if let Ok(value) = env::var("SI_TEST_NATS_URL") {
            config.url = value;
        }
        config
    }

    async fn nats() -> NatsClient {
        NatsClient::new(&nats_config())
            .await
            .expect("failed to connect to NATS")
    }

    fn nats_prefix() -> String {
        Uuid::new_v4().as_simple().to_string()
    }

    async fn veritech_server_for_uds_cyclone(subject_prefix: String) -> Server {
        let cyclone_spec = CycloneSpec::LocalUds(
            LocalUdsInstance::spec()
                .try_cyclone_cmd_path("../../target/debug/cyclone")
                .expect("failed to setup cyclone_cmd_path")
                .cyclone_decryption_key_path("../../lib/cyclone-server/src/dev.decryption.key")
                .try_lang_server_cmd_path("../../bin/lang-js/target/lang-js")
                .expect("failed to setup lang_js_cmd_path")
                .all_endpoints()
                .build()
                .expect("failed to build cyclone spec"),
        );
        let config = Config::builder()
            .nats(nats_config())
            .subject_prefix(subject_prefix)
            .cyclone_spec(cyclone_spec)
            .build()
            .expect("failed to build spec");
        Server::for_cyclone_uds(config)
            .await
            .expect("failed to create server")
    }

    async fn client(subject_prefix: String) -> Client {
        Client::with_subject_prefix(nats().await, subject_prefix)
    }

    async fn run_veritech_server_for_uds_cyclone(
        subject_prefix: String,
    ) -> JoinHandle<Result<(), ServerError>> {
        tokio::spawn(veritech_server_for_uds_cyclone(subject_prefix).await.run())
    }

    #[test(tokio::test)]
    async fn executes_simple_resolver_function() {
        let prefix = nats_prefix();
        run_veritech_server_for_uds_cyclone(prefix.clone()).await;
        let client = client(prefix).await;

        // Not going to check output here--we aren't emitting anything
        let (tx, mut rx) = mpsc::channel(64);
        tokio::spawn(async move {
            while let Some(output) = rx.recv().await {
                info!("output: {:?}", output)
            }
        });

        let request = ResolverFunctionRequest {
            execution_id: "1234".to_string(),
            handler: "numberOfInputs".to_string(),
            component: ResolverFunctionComponent {
                data: ComponentView {
                    properties: serde_json::json!({ "foo": "bar", "baz": "quux" }),
                    system: None,
                    kind: ComponentKind::Standard,
                    resource: None,
                },
                parents: vec![],
            },
            code_base64: base64::encode(
                "function numberOfInputs(input) { return Object.keys(input).length; }",
            ),
        };

        let result = client
            .execute_resolver_function(tx, &request)
            .await
            .expect("failed to execute resolver function");

        match result {
            FunctionResult::Success(success) => {
                assert_eq!(success.execution_id, "1234");
                assert_eq!(success.data, serde_json::json!(2));
                assert!(!success.unset);
            }
            FunctionResult::Failure(failure) => {
                panic!("function did not succeed and should have: {:?}", failure)
            }
        }
    }

    #[test(tokio::test)]
    async fn executes_simple_qualification_check() {
        let prefix = nats_prefix();
        run_veritech_server_for_uds_cyclone(prefix.clone()).await;
        let client = client(prefix).await;

        // Not going to check output here--we aren't emitting anything
        let (tx, mut rx) = mpsc::channel(64);
        tokio::spawn(async move {
            while let Some(output) = rx.recv().await {
                info!("output: {:?}", output)
            }
        });

        let mut request = QualificationCheckRequest {
            execution_id: "5678".to_string(),
            handler: "check".to_string(),
            component: QualificationCheckComponent {
                data: ComponentView {
                    properties: serde_json::json!({"image": "systeminit/whiskers"}),
                    system: None,
                    kind: ComponentKind::Standard,
                    resource: None,
                },
                codes: vec![CodeGenerated {
                    format: "yaml".to_owned(),
                    code: "generateName: asd\nname: kubernetes_deployment\napiVersion: apps/v1\nkind: Deployment\n".to_owned()
                }],
                parents: Vec::new(),
            },
            code_base64: base64::encode(indoc! {r#"
                async function check(component) {
                    const skopeoChild = await siExec.waitUntilEnd("skopeo", ["inspect", `docker://docker.io/${component.data.properties.image}`]);

                    const code = component.codes[0];
                    const file = path.join(os.tmpdir(), "veritech-kubeval-test.yaml");
                    fs.writeFileSync(file, code.code);

                    try {
                        const child = await siExec.waitUntilEnd("kubeval", [file]);

                        return {
                          qualified: skopeoChild.exitCode === 0 && child.exitCode === 0,
                          message: JSON.stringify({ skopeoStdout: skopeoChild.stdout, skopeoStderr: skopeoChild.stderr, kubevalStdout: child.stdout, kubevalStderr: child.stderr }),
                        };
                    } finally {
                        fs.unlinkSync(file);
                    }
                }
            "#}),
        };

        // Run a qualified check (i.e. qualification returns qualified == true)
        let result = client
            .execute_qualification_check(tx.clone(), &request)
            .await
            .expect("failed to execute qualification check");

        match result {
            FunctionResult::Success(success) => {
                assert_eq!(success.execution_id, "5678");
                // Note: this might be fragile, as skopeo stdout API might change (?)
                let message = success.message.expect("no message available");
                assert_eq!(
                    serde_json::from_str::<serde_json::Value>(
                        serde_json::from_str::<serde_json::Value>(&message,)
                            .expect("Message is not json")
                            .as_object()
                            .expect("Message isn't an object")
                            .get("skopeoStdout")
                            .expect("Key skopeoStdout wasn't found")
                            .as_str()
                            .expect("skopeoStdout is not a string")
                    )
                    .expect("skopeoStdout is not json")
                    .as_object()
                    .expect("skopeoStdout isn't an object")
                    .get("Name")
                    .expect("Key Name wasn't found")
                    .as_str(),
                    Some("docker.io/systeminit/whiskers")
                );
                assert_eq!(
                    serde_json::from_str::<serde_json::Value>(&message,)
                        .expect("Message is not json")
                        .as_object()
                        .expect("Message isn't an object")
                        .get("skopeoStderr")
                        .expect("Key skopeoStderr wasn't found")
                        .as_str(),
                    Some("")
                );
                assert_eq!(
                    serde_json::from_str::<serde_json::Value>(&message,)
                        .expect("Message is not json")
                        .as_object()
                        .expect("Message isn't an object")
                        .get("kubevalStdout")
                        .expect("Key kubevalStdout wasn't found")
                        .as_str(),
                    Some(
                        format!(
                            "PASS - {} contains a valid Deployment (unknown)",
                            std::env::temp_dir()
                                .join("veritech-kubeval-test.yaml")
                                .display()
                        )
                        .as_str()
                    )
                );
                assert_eq!(
                    serde_json::from_str::<serde_json::Value>(&message,)
                        .expect("Message is not json")
                        .as_object()
                        .expect("Message isn't an object")
                        .get("kubevalStderr")
                        .expect("Key kubevalStderr wasn't found")
                        .as_str(),
                    Some("")
                );
                assert!(success.qualified);
            }
            FunctionResult::Failure(failure) => {
                panic!("function did not succeed and should have: {:?}", failure)
            }
        }

        request.execution_id = "9012".to_string();
        request.component.data = ComponentView {
            properties: serde_json::json!({"image": "abc"}),
            system: None,
            kind: ComponentKind::Standard,
            resource: None,
        };

        // Now update the request to re-run an unqualified check (i.e. qualification returning
        // qualified == false)
        let result = client
            .execute_qualification_check(tx, &request)
            .await
            .expect("failed to execute qualification check");

        match result {
            FunctionResult::Success(success) => {
                assert_eq!(success.execution_id, "9012");
                assert!(!success.qualified);
            }
            FunctionResult::Failure(failure) => {
                panic!("function did not succeed and should have: {:?}", failure)
            }
        }
    }

    #[test(tokio::test)]
    async fn executes_simple_confirmation() {
        let prefix = nats_prefix();
        run_veritech_server_for_uds_cyclone(prefix.clone()).await;
        let client = client(prefix).await;

        // Not going to check output here--we aren't emitting anything
        let (tx, mut rx) = mpsc::channel(64);
        tokio::spawn(async move {
            while let Some(output) = rx.recv().await {
                info!("output: {:?}", output)
            }
        });

        let request = ConfirmationRequest {
            execution_id: "7868".to_string(),
            handler: "confirmItOut".to_string(),
            component: ComponentView {
                properties: serde_json::json!({"pkg": "cider"}),
                system: None,
                kind: ComponentKind::Standard, resource: None,
            },
            code_base64: base64::encode("function confirmItOut(component) { return { success: true, recommendedActions: ['vai te catar'] } }")
        };

        let result = client
            .execute_confirmation(tx, &request)
            .await
            .expect("failed to execute confirmation");

        match result {
            FunctionResult::Success(success) => {
                assert_eq!(success.execution_id, "7868");
                assert!(success.success);
                assert_eq!(success.recommended_actions, vec!["vai te catar".to_owned()]);
            }
            FunctionResult::Failure(failure) => {
                panic!("function did not succeed and should have: {:?}", failure)
            }
        }
    }

    #[test(tokio::test)]
    async fn executes_simple_code_generation() {
        let prefix = nats_prefix();
        run_veritech_server_for_uds_cyclone(prefix.clone()).await;
        let client = client(prefix).await;

        // Not going to check output here--we aren't emitting anything
        let (tx, mut rx) = mpsc::channel(64);
        tokio::spawn(async move {
            while let Some(output) = rx.recv().await {
                info!("output: {:?}", output)
            }
        });

        let request = CodeGenerationRequest {
            execution_id: "7868".to_string(),
            handler: "generateItOut".to_string(),
            component: ComponentView {
                properties: serde_json::json!({"pkg": "cider"}),
                system: None,
                kind: ComponentKind::Standard,
                    resource: None,
            },
            code_base64: base64::encode("function generateItOut(component) { return { format: 'yaml', code: YAML.stringify(component.properties) }; }"),
        };

        let result = client
            .execute_code_generation(tx, &request)
            .await
            .expect("failed to execute code generation");

        match result {
            FunctionResult::Success(success) => {
                assert_eq!(success.execution_id, "7868");
                assert_eq!(
                    success.data,
                    CodeGenerated {
                        format: "yaml".to_owned(),
                        code: "pkg: cider\n".to_owned(),
                    }
                );
            }
            FunctionResult::Failure(failure) => {
                panic!("function did not succeed and should have: {:?}", failure)
            }
        }
    }

    #[test(tokio::test)]
    async fn executes_simple_validation() {
        let prefix = nats_prefix();
        run_veritech_server_for_uds_cyclone(prefix.clone()).await;
        let client = client(prefix).await;

        // Not going to check output here--we aren't emitting anything
        let (tx, mut rx) = mpsc::channel(64);
        tokio::spawn(async move {
            while let Some(output) = rx.recv().await {
                info!("output: {:?}", output)
            }
        });

        let request = ValidationRequest {
            execution_id: "31337".to_string(),
            handler: "isThirtyThree".to_string(),
            value: 33.into(),
            code_base64: base64::encode(
                "function isThirtyThree(value) { return { valid: value === 33 }; };",
            ),
        };

        let result = client
            .execute_validation(tx, &request)
            .await
            .expect("failed to execute validation");

        match result {
            FunctionResult::Success(success) => {
                assert_eq!(success.execution_id, "31337");
                assert!(success.valid);
            }
            FunctionResult::Failure(failure) => {
                panic!("function did not succeed and should have: {:?}", failure)
            }
        }
    }

    #[test(tokio::test)]
    async fn executes_simple_workflow_resolve() {
        let prefix = nats_prefix();
        run_veritech_server_for_uds_cyclone(prefix.clone()).await;
        let client = client(prefix).await;

        // Not going to check output here--we aren't emitting anything
        let (tx, mut rx) = mpsc::channel(64);
        tokio::spawn(async move {
            while let Some(output) = rx.recv().await {
                info!("output: {:?}", output)
            }
        });

        let request = WorkflowResolveRequest {
            execution_id: "112233".to_string(),
            handler: "workItOut".to_string(),
            // TODO(fnichol): rewrite this function once we settle on contract
            code_base64: base64::encode("function workItOut() { return { name: 'mc fioti', kind: 'vacina butantan - https://www.youtube.com/watch?v=yQ8xJHuW7TY', steps: [] }; }"),
            args: Default::default(),
        };

        let result = client
            .execute_workflow_resolve(tx, &request)
            .await
            .expect("failed to execute workflow resolve");

        match result {
            FunctionResult::Success(success) => {
                assert_eq!(success.execution_id, "112233");
                // TODO(fnichol): add more assertions as we add fields
            }
            FunctionResult::Failure(failure) => {
                panic!("function did not succeed and should have: {:?}", failure)
            }
        }
    }
}
