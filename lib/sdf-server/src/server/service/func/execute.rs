use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{
    func::binding::FuncBindingResult, func::binding::LogLinePayload, DalContext, Func, FuncBinding,
    FuncBindingError, FuncError, FuncId, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};
use veritech_client::OutputStream;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteRequest {
    pub id: FuncId,
    pub args: serde_json::Value,
    pub execution_key: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteResponse {
    pub id: FuncId,
    pub args: serde_json::Value,
    pub output: serde_json::Value,
    pub execution_key: String,
    pub logs: Vec<OutputStream>,
}

pub async fn execute(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(req): Json<ExecuteRequest>,
) -> FuncResult<Json<ExecuteResponse>> {
    let ctx = builder.build(request_ctx.build(req.visibility)).await?;

    let func = Func::get_by_id(&ctx, &req.id)
        .await?
        .ok_or(FuncError::NotFound(req.id))?;
    let func_binding =
        FuncBinding::new(&ctx, req.args.clone(), req.id, *func.backend_kind()).await?;

    let (func, _execution, context, mut rx) = func_binding.prepare_execution(&ctx).await?;
    ctx.rollback().await?;

    // Doesn't use transaction in ctx
    let (func_id, inner_ctx, execution_key) = (*func.id(), ctx.clone(), req.execution_key.clone());
    let log_handler = tokio::spawn(async move {
        let (ctx, mut output) = (&inner_ctx, Vec::new());

        while let Some(output_stream) = rx.recv().await {
            output.push(output_stream.clone());

            let log_line = LogLinePayload {
                line: output_stream,
                func_id,
                execution_key: execution_key.clone(),
            };
            publish_immediately(ctx, WsEvent::log_line(ctx, log_line).await?).await?;
        }
        Ok::<_, FuncBindingError>(output)
    });

    let (value, _unprocessed_value) = func_binding
        .execute_critical_section(func.clone(), context)
        .await?;
    let logs = log_handler.await??;

    Ok(Json(ExecuteResponse {
        id: req.id,
        args: req.args,
        execution_key: req.execution_key,
        output: value.unwrap_or(serde_json::Value::Null),
        logs,
    }))
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
///  because [`events`](crate::WsEvent) should likely not be reused.
async fn publish_immediately(ctx: &DalContext, ws_event: WsEvent) -> FuncBindingResult<()> {
    let subject = format!("si.workspace_pk.{}.event", ws_event.workspace_pk());
    let msg_bytes = serde_json::to_vec(&ws_event)?;
    ctx.nats_conn().publish(subject, msg_bytes).await?;
    Ok(())
}
