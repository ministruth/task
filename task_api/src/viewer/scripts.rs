use skynet_api::{
    HyUuid, Result, anyhow,
    hyuuid::uuids2strings,
    request::Condition,
    sea_orm::{
        self, ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, ConnectionTrait, EntityTrait,
        PaginatorTrait, QueryFilter, Set, Unchanged,
    },
};
use skynet_macro::default_viewer;

use crate::entity::scripts;

pub struct ScriptViewer;

#[default_viewer(scripts)]
impl ScriptViewer {
    pub async fn create<C>(db: &C, name: &str, code: &str) -> Result<scripts::Model>
    where
        C: ConnectionTrait,
    {
        scripts::ActiveModel {
            name: Set(name.to_owned()),
            code: Set(code.to_owned()),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(Into::into)
    }

    pub async fn update<C>(
        db: &C,
        id: &HyUuid,
        name: Option<&str>,
        code: Option<&str>,
    ) -> Result<scripts::Model>
    where
        C: ConnectionTrait,
    {
        scripts::ActiveModel {
            id: Unchanged(*id),
            name: name.map_or(NotSet, |x| Set(x.to_owned())),
            code: code.map_or(NotSet, |x| Set(x.to_owned())),
            ..Default::default()
        }
        .update(db)
        .await
        .map_err(Into::into)
    }
}
