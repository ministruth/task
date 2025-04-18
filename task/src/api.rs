use actix_cloud::{
    actix_web::web::{Data, Path},
    response::{JsonResponse, RspResult},
    tracing::info,
};
use actix_web_validator::{Json, QsQuery};
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use skynet_api::{
    HyUuid,
    ffi_rpc::registry::Registry,
    finish,
    request::{Condition, IDsReq, IntoExpr, PageData, PaginationParam, TimeParam},
    sea_orm::{ColumnTrait, IntoSimpleExpr, TransactionTrait},
};
use skynet_api_task::{
    Service,
    entity::{scripts, tasks},
    viewer::{scripts::ScriptViewer, tasks::TaskViewer},
};
use skynet_macro::common_req;
use validator::Validate;

use crate::{PLUGIN_INSTANCE, TaskResponse};

#[common_req(tasks::Column)]
#[derive(Debug, Validate, Deserialize)]
pub struct GetTasksReq {
    pub text: Option<String>,

    #[serde(flatten)]
    #[validate(nested)]
    pub page: PaginationParam,
    #[serde(flatten)]
    #[validate(nested)]
    pub time: TimeParam,
}

pub async fn get_tasks(param: QsQuery<GetTasksReq>) -> RspResult<JsonResponse> {
    let mut cond = param.common_cond();
    if let Some(text) = &param.text {
        cond = cond.add(
            Condition::any()
                .add(text.like_expr(tasks::Column::Id))
                .add(text.like_expr(tasks::Column::Name))
                .add(text.like_expr(tasks::Column::Detail))
                .add(text.like_expr(tasks::Column::Output)),
        );
    }
    let data = TaskViewer::find(PLUGIN_INSTANCE.db.get().unwrap(), cond).await?;
    finish!(JsonResponse::new(TaskResponse::Success).json(PageData::new(data)));
}

#[serde_inline_default]
#[derive(Debug, Validate, Deserialize)]
pub struct GetOutputReq {
    #[validate(range(min = 0))]
    #[serde_inline_default(0)]
    pub pos: usize,
}

pub async fn get_output(
    tid: Path<HyUuid>,
    param: QsQuery<GetOutputReq>,
) -> RspResult<JsonResponse> {
    #[derive(Serialize)]
    struct Rsp {
        output: String,
        pos: usize,
    }
    let t = match TaskViewer::find_by_id(PLUGIN_INSTANCE.db.get().unwrap(), &tid).await? {
        Some(t) => t.output,
        None => finish!(JsonResponse::not_found()),
    }
    .unwrap_or_default();

    let t = if param.pos < t.len() {
        &t[param.pos..]
    } else {
        ""
    };
    finish!(JsonResponse::new(TaskResponse::Success).json(Rsp {
        output: t.to_owned(),
        pos: param.pos + t.len()
    }));
}

pub async fn delete_completed() -> RspResult<JsonResponse> {
    let cnt = TaskViewer::delete_completed(PLUGIN_INSTANCE.db.get().unwrap()).await?;
    info!(success = true, "Delete all tasks");
    finish!(JsonResponse::new(TaskResponse::Success).json(cnt));
}

pub async fn stop(tid: Path<HyUuid>, reg: Data<Registry>) -> RspResult<JsonResponse> {
    if !PLUGIN_INSTANCE.stop(&reg, *tid).await {
        finish!(JsonResponse::not_found());
    }
    info!(success = true, id = %tid, "Stop task");
    finish!(JsonResponse::new(TaskResponse::Success));
}

#[common_req(scripts::Column)]
#[derive(Debug, Validate, Deserialize)]
pub struct GetScriptsReq {
    pub text: Option<String>,

    #[serde(flatten)]
    #[validate(nested)]
    pub page: PaginationParam,
    #[serde(flatten)]
    #[validate(nested)]
    pub time: TimeParam,
}

pub async fn get_scripts(param: QsQuery<GetScriptsReq>) -> RspResult<JsonResponse> {
    #[derive(Serialize)]
    struct Rsp {
        id: HyUuid,
        name: String,
        created_at: i64,
        updated_at: i64,
    }
    let mut cond = param.common_cond();
    if let Some(text) = &param.text {
        cond = cond.add(
            Condition::any()
                .add(text.like_expr(scripts::Column::Id))
                .add(text.like_expr(scripts::Column::Name)),
        );
    }
    let data = ScriptViewer::find(PLUGIN_INSTANCE.db.get().unwrap(), cond).await?;
    let data = (
        data.0
            .into_iter()
            .map(|x| Rsp {
                id: x.id,
                name: x.name,
                created_at: x.created_at,
                updated_at: x.updated_at,
            })
            .collect(),
        data.1,
    );
    finish!(JsonResponse::new(TaskResponse::Success).json(PageData::new(data)));
}

pub async fn get_script(sid: Path<HyUuid>) -> RspResult<JsonResponse> {
    if let Some(script) = ScriptViewer::find_by_id(PLUGIN_INSTANCE.db.get().unwrap(), &sid).await? {
        finish!(JsonResponse::new(TaskResponse::Success).json(script));
    } else {
        finish!(JsonResponse::not_found());
    }
}

#[derive(Debug, Validate, Deserialize)]
pub struct AddScriptReq {
    #[validate(length(min = 1, max = 32))]
    pub name: String,
    pub code: String,
}

pub async fn add_script(param: Json<AddScriptReq>) -> RspResult<JsonResponse> {
    let script =
        ScriptViewer::create(PLUGIN_INSTANCE.db.get().unwrap(), &param.name, &param.code).await?;
    info!(success = true, name = param.name, "Add script");
    finish!(JsonResponse::new(TaskResponse::Success).json(script.id));
}

#[derive(Debug, Validate, Deserialize)]
pub struct PutScriptReq {
    #[validate(length(min = 1, max = 32))]
    pub name: Option<String>,
    pub code: Option<String>,
}

pub async fn put_script(sid: Path<HyUuid>, param: Json<PutScriptReq>) -> RspResult<JsonResponse> {
    let tx = PLUGIN_INSTANCE.db.get().unwrap().begin().await?;
    if let Some(script) = ScriptViewer::find_by_id(&tx, &sid).await? {
        ScriptViewer::update(
            &tx,
            &script.id,
            param.name.as_deref(),
            param.code.as_deref(),
        )
        .await?;
    } else {
        finish!(JsonResponse::not_found());
    }
    tx.commit().await?;
    info!(
        success = true,
        sid = %sid,
        "Put script",
    );
    finish!(JsonResponse::new(TaskResponse::Success));
}

pub async fn delete_script_batch(param: Json<IDsReq>) -> RspResult<JsonResponse> {
    let rows = ScriptViewer::delete(PLUGIN_INSTANCE.db.get().unwrap(), &param.id).await?;
    if rows != 0 {
        info!(
            success = true,
            sid = ?param.id,
            "Delete scripts",
        );
    }
    finish!(JsonResponse::new(TaskResponse::Success).json(rows));
}

pub async fn delete_script(sid: Path<HyUuid>) -> RspResult<JsonResponse> {
    let tx = PLUGIN_INSTANCE.db.get().unwrap().begin().await?;
    if ScriptViewer::find_by_id(&tx, &sid).await?.is_none() {
        finish!(JsonResponse::not_found());
    }
    let rows = ScriptViewer::delete(&tx, &[*sid]).await?;
    tx.commit().await?;
    info!(
        success = true,
        sid = %sid,
        "Delete script",
    );
    finish!(JsonResponse::new(TaskResponse::Success).json(rows));
}

pub async fn run_script(sid: Path<HyUuid>, reg: Data<Registry>) -> RspResult<JsonResponse> {
    if let Some(s) = ScriptViewer::find_by_id(PLUGIN_INSTANCE.db.get().unwrap(), &sid).await? {
        let ret = PLUGIN_INSTANCE
            .create_code(&reg, format!("Manual run `{}`", s.name), None, s.code)
            .await?;
        info!(
            success = true,
            sid = %sid,
            "Run script",
        );
        finish!(JsonResponse::new(TaskResponse::Success).json(ret));
    } else {
        finish!(JsonResponse::not_found());
    }
}
