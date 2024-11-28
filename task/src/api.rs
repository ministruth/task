use actix_cloud::{
    actix_web::web::{Data, Path},
    response::{JsonResponse, RspResult},
    tracing::info,
};
use actix_web_validator::QsQuery;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use skynet_api::{
    ffi_rpc::registry::Registry,
    finish,
    request::{Condition, IntoExpr, PageData, PaginationParam, TimeParam},
    sea_orm::{ColumnTrait, IntoSimpleExpr},
    HyUuid,
};
use skynet_api_task::{entity::tasks, viewer::tasks::TaskViewer, Service};
use skynet_macro::common_req;
use validator::Validate;

use crate::{TaskResponse, PLUGIN_INSTANCE};

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

pub async fn get_all(param: QsQuery<GetTasksReq>) -> RspResult<JsonResponse> {
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
