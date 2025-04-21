use actix_cloud::chrono;
use actix_cloud::macros::{entity_behavior, entity_id, entity_timestamp};
use serde::{Deserialize, Serialize};
use skynet_api::sea_orm::{self, prelude::*};

use crate::HyUuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Default, Serialize, Deserialize)]
#[sea_orm(table_name = "4adaf7d3-b877-43c3-82bd-da3689dc3920_scripts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: HyUuid,
    pub name: String,
    pub code: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::tasks::Entity")]
    Task,
}

impl Related<super::tasks::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

#[entity_id(HyUuid::new())]
#[entity_timestamp]
impl ActiveModel {}

#[entity_behavior]
impl ActiveModelBehavior for ActiveModel {}
