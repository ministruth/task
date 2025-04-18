use skynet_api::{
    HyUuid, Result, anyhow,
    hyuuid::uuids2strings,
    request::Condition,
    sea_orm::{
        self, ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseTransaction, EntityTrait,
        IntoActiveModel, PaginatorTrait, QueryFilter, Set, Unchanged, prelude::Expr,
    },
};
use skynet_macro::default_viewer;

use crate::entity::tasks;

pub struct TaskViewer;

#[default_viewer(tasks)]
impl TaskViewer {
    pub async fn create<C>(db: &C, name: &str, detail: &Option<String>) -> Result<tasks::Model>
    where
        C: ConnectionTrait,
    {
        tasks::ActiveModel {
            name: Set(name.to_owned()),
            detail: Set(detail.to_owned()),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(Into::into)
    }

    /// Update task `id` with `output` and `percent`.
    pub async fn update(
        db: &DatabaseTransaction,
        id: &HyUuid,
        output: &str,
        percent: u32,
    ) -> Result<bool> {
        let mut m = match Self::find_by_id(db, id).await? {
            Some(x) => x,
            None => return Ok(false),
        };
        let output = m.output.take().unwrap_or_default() + output;
        let percent = m.percent.saturating_add(percent.try_into()?).min(100);
        let mut m: tasks::ActiveModel = m.into();
        m.output = Set(Some(output));
        m.percent = Set(percent);
        m.update(db).await?;
        Ok(true)
    }

    pub async fn finish_out<C>(db: &C, id: &HyUuid, result: i32, output: &str) -> Result<bool>
    where
        C: ConnectionTrait,
    {
        let mut m = match Self::find_by_id(db, id).await? {
            Some(x) => x,
            None => return Ok(false),
        };
        let output = m.output.take().unwrap_or_default() + output;
        let mut m = m.into_active_model();
        m.result = Set(Some(result));
        m.output = Set(Some(output));
        m.update(db).await?;
        Ok(true)
    }

    /// Finish task `id` with `result`.
    pub async fn finish<C>(db: &C, id: &HyUuid, result: i32) -> Result<()>
    where
        C: ConnectionTrait,
    {
        tasks::ActiveModel {
            id: Unchanged(*id),
            result: Set(Some(result)),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(())
    }

    /// Delete all completed tasks.
    pub async fn delete_completed<C>(db: &C) -> Result<u64>
    where
        C: ConnectionTrait,
    {
        tasks::Entity::delete_many()
            .filter(tasks::Column::Result.is_not_null())
            .exec(db)
            .await
            .map(|x| x.rows_affected)
            .map_err(Into::into)
    }

    /// Clean all running tasks, mark result to unknown.
    pub async fn clean_running<C>(db: &C) -> Result<u64>
    where
        C: ConnectionTrait,
    {
        Ok(tasks::Entity::update_many()
            .col_expr(tasks::Column::Result, Expr::value(-1))
            .filter(tasks::Column::Result.is_null())
            .exec(db)
            .await?
            .rows_affected)
    }
}
