use super::{SchemaVariantDefinitionError, SchemaVariantDefinitionResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::Json;
use dal::func::intrinsics::IntrinsicFunc;
use dal::installed_pkg::{InstalledPkgAsset, InstalledPkgAssetAssetId, InstalledPkgAssetKind};
use dal::pkg::import_pkg_from_pkg;
use dal::{
    schema::variant::definition::{
        SchemaVariantDefinition, SchemaVariantDefinitionId, SchemaVariantDefinitionJson,
        SchemaVariantDefinitionMetadataJson,
    },
    Func, FuncBinding, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};
use si_pkg::{FuncSpec, FuncSpecBackendKind, FuncSpecBackendResponseType, PkgSpec, SiPkg};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecVariantDefRequest {
    pub id: SchemaVariantDefinitionId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

// Should move this to the modules service
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstalledPkgAssetView {
    pub asset_id: InstalledPkgAssetAssetId,
    pub asset_kind: InstalledPkgAssetKind,
    pub asset_hash: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecVariantDefResponse {
    pub success: bool,
    pub installed_pkg_assets: Vec<InstalledPkgAssetView>,
    pub func_exec_response: serde_json::Value,
}

pub async fn exec_variant_def(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<ExecVariantDefRequest>,
) -> SchemaVariantDefinitionResult<Json<ExecVariantDefResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let variant_def = SchemaVariantDefinition::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(SchemaVariantDefinitionError::VariantDefinitionNotFound(
            request.id,
        ))?;

    let asset_func = Func::get_by_id(&ctx, &variant_def.func_id()).await?.ok_or(
        SchemaVariantDefinitionError::FuncNotFound(variant_def.func_id()),
    )?;

    let metadata: SchemaVariantDefinitionMetadataJson = variant_def.clone().into();

    let (_, return_value) =
        FuncBinding::create_and_execute(&ctx, serde_json::Value::Null, *asset_func.id()).await?;

    let func_resp = return_value
        .value()
        .ok_or(SchemaVariantDefinitionError::FuncExecution(
            *asset_func.id(),
        ))?
        .as_object()
        .ok_or(SchemaVariantDefinitionError::FuncExecution(
            *asset_func.id(),
        ))?
        .get("definition")
        .ok_or(SchemaVariantDefinitionError::FuncExecution(
            *asset_func.id(),
        ))?;
    let definition: SchemaVariantDefinitionJson = serde_json::from_value(func_resp.to_owned())?;

    // we need to change this to use the PkgImport
    let identity_func_spec = IntrinsicFunc::Identity.to_spec()?;

    let mut schema_variant_func_spec = FuncSpec::builder();
    schema_variant_func_spec.name(String::from(asset_func.name()));
    schema_variant_func_spec.backend_kind(FuncSpecBackendKind::JsSchemaVariantDefinition);
    schema_variant_func_spec.response_type(FuncSpecBackendResponseType::SchemaVariantDefinition);
    schema_variant_func_spec.hidden(asset_func.hidden());
    if let Some(code) = asset_func.code_plaintext()? {
        schema_variant_func_spec.code_plaintext(code);
    }
    if let Some(handler) = asset_func.handler() {
        schema_variant_func_spec.handler(handler.to_string());
    }
    if let Some(description) = asset_func.description() {
        schema_variant_func_spec.description(description.to_string());
    }
    if let Some(display_name) = asset_func.display_name() {
        schema_variant_func_spec.display_name(display_name.to_string());
    }
    let asset_func_built = schema_variant_func_spec.build()?;

    let variant_spec = definition.to_spec(
        metadata.clone(),
        identity_func_spec.unique_id,
        asset_func_built.unique_id,
    )?;
    let schema_spec = metadata.to_spec(variant_spec)?;
    let pkg_spec = PkgSpec::builder()
        .name(metadata.clone().name)
        .created_by("sally@systeminit.com")
        .func(identity_func_spec)
        .func(asset_func_built.clone())
        .schema(schema_spec)
        .version("0.0.1")
        .build()?;

    let pkg = SiPkg::load_from_spec(pkg_spec.clone())?;
    let installed_pkg_id = import_pkg_from_pkg(
        &ctx,
        &pkg,
        metadata.clone().name.as_str(),
        Some(dal::pkg::ImportOptions {
            schemas: None,
            no_definitions: true,
            skip_import_funcs: Some(HashMap::from_iter([(
                asset_func_built.unique_id,
                asset_func.clone(),
            )])),
        }),
    )
    .await?;

    let installed_pkg_assets =
        InstalledPkgAsset::list_for_installed_pkg_id(&ctx, installed_pkg_id).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "exec_variant_def",
        serde_json::json!({
                    "variant_def_category": metadata.clone().category,
                    "variant_def_name": metadata.clone().name,
                    "variant_def_version": pkg_spec.clone().version,
                    "variant_def_schema_count":  pkg_spec.clone().schemas.len(),
                    "variant_def_function_count":  pkg_spec.clone().funcs.len(),
        }),
    );

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;
    ctx.commit().await?;

    Ok(Json(ExecVariantDefResponse {
        success: true,
        func_exec_response: func_resp.to_owned(),
        installed_pkg_assets: installed_pkg_assets
            .iter()
            .map(|ipa| InstalledPkgAssetView {
                asset_id: ipa.asset_id(),
                asset_hash: ipa.asset_hash().into(),
                asset_kind: ipa.asset_kind().to_owned(),
            })
            .collect(),
    }))
}
