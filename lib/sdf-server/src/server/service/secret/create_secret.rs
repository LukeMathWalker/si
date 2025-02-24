use axum::Json;
use dal::{
    key_pair::KeyPairPk, EncryptedSecret, Secret, SecretAlgorithm, SecretKind, SecretObjectType,
    SecretVersion, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext};

use super::SecretResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSecretRequest {
    pub name: String,
    pub object_type: SecretObjectType,
    pub kind: SecretKind,
    pub crypted: Vec<u8>,
    pub key_pair_pk: KeyPairPk,
    pub version: SecretVersion,
    pub algorithm: SecretAlgorithm,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSecretResponse {
    pub secret: Secret,
}

pub async fn create_secret(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_tx): AccessBuilder,
    Json(request): Json<CreateSecretRequest>,
) -> SecretResult<Json<CreateSecretResponse>> {
    let ctx = builder.build(request_tx.build(request.visibility)).await?;

    let secret = EncryptedSecret::new(
        &ctx,
        request.name,
        request.object_type,
        request.kind,
        &request.crypted,
        request.key_pair_pk,
        request.version,
        request.algorithm,
    )
    .await?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(CreateSecretResponse { secret }))
}
