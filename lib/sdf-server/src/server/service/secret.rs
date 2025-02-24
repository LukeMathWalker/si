use std::collections::HashMap;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::Json;
use axum::Router;
use chrono::Utc;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;

use dal::diagram::node::HistoryEventMetadata;
use dal::{
    DiagramError, HistoryActorTimestamp, KeyPairError, SecretId, StandardModelError,
    TransactionsError, UserError, Visibility, WorkspacePk, WsEventError,
};

use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::server::state::AppState;

pub mod get_public_key;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SecretError {
    #[error(transparent)]
    ContextTransactions(#[from] TransactionsError),
    #[error(transparent)]
    Diagram(#[from] DiagramError),
    #[error(transparent)]
    KeyPairError(#[from] KeyPairError),
    #[error(transparent)]
    Nats(#[from] si_data_nats::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    Secret(#[from] dal::SecretError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    User(#[from] UserError),
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(WorkspacePk),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type SecretResult<T> = std::result::Result<T, SecretError>;

// NOTE(victor): This is a temporary struct created only for the static array storage to work
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Secret {
    id: SecretId,
    name: String,
    description: Option<String>,
    definition: String,
    created_info: Option<HistoryEventMetadata>,
    updated_info: Option<HistoryEventMetadata>,
}

// TODO(victor): Remove this as soon as we can store secrets for real
static SECRETOS: Lazy<Mutex<HashMap<String, Vec<Secret>>>> = Lazy::new(Mutex::default);

impl IntoResponse for SecretError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());
        //SecretError::SecretNotFound => (StatusCode::NOT_FOUND, self.to_string()),

        let body = Json(serde_json::json!({
            "error": {
                "message": error_message,
                "code": 42,
                "statusCode": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/get_public_key", get(get_public_key::get_public_key))
        .route("/", post(create_secret))
        .route("/", get(list_secrets))
        .route("/", delete(delete_secrets))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSecretRequest {
    name: String,
    description: Option<String>,
    definition: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type CreateSecretResponse = Secret;

pub async fn create_secret(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Json(request): Json<CreateSecretRequest>,
) -> SecretResult<Json<CreateSecretResponse>> {
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    let ctx = builder.build_head(access_builder).await?;

    let created_info = HistoryActorTimestamp {
        actor: *access_builder.history_actor(),
        timestamp: Utc::now(),
    };

    let id = SecretId::generate();

    let secret = Secret {
        id,
        name: request.name,
        description: request.description,
        definition: request.definition.clone(),
        created_info: Some(
            HistoryEventMetadata::from_history_actor_timestamp(&ctx, created_info).await?,
        ),
        updated_info: None,
    };

    SECRETOS
        .lock()
        .await
        .entry(request.definition)
        .or_default()
        .push(secret.clone());

    Ok(Json(secret))
}

pub type ListSecretResponse = HashMap<String, Vec<Secret>>;

pub async fn list_secrets() -> SecretResult<Json<ListSecretResponse>> {
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    Ok(Json((*SECRETOS.lock().await).clone()))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSecretRequest {
    id: SecretId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn delete_secrets(Json(request): Json<DeleteSecretRequest>) -> SecretResult<Json<()>> {
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    SECRETOS
        .lock()
        .await
        .values_mut()
        .for_each(|v| v.retain(|secret| secret.id != request.id));

    Ok(Json(()))
}
