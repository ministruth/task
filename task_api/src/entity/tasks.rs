use actix_cloud::chrono;
use actix_cloud::macros::{entity_behavior, entity_id, entity_timestamp};
use serde::{Deserialize, Serialize};
use skynet_api::sea_orm::{self, prelude::*};

use crate::HyUuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Default, Serialize, Deserialize)]
#[sea_orm(table_name = "4adaf7d3-b877-43c3-82bd-da3689dc3920_tasks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: HyUuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip)]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<i32>,
    pub sid: Option<HyUuid>,
    pub percent: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::scripts::Entity",
        from = "Column::Sid",
        to = "super::scripts::Column::Id"
    )]
    Script,
}

impl Related<super::scripts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Script.def()
    }
}

#[entity_id(HyUuid::new())]
#[entity_timestamp]
impl ActiveModel {}

#[entity_behavior]
impl ActiveModelBehavior for ActiveModel {}
