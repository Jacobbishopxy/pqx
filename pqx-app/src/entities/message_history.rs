//! file: message_history.rs
//! author: Jacob Xie
//! date: 2023/06/13 23:05:30 Tuesday
//! brief:

use sea_orm::entity::prelude::*;

// ================================================================================================
// Model: message_history
// ================================================================================================

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "message_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub consumer_ids: String,
    #[sea_orm(nullable)]
    pub retry: Option<i16>,
    #[sea_orm(nullable)]
    pub poke: Option<i32>,
    #[sea_orm(nullable)]
    pub waiting_timeout: Option<i64>,
    #[sea_orm(nullable)]
    pub consuming_timeout: Option<i64>,
    pub cmd: Json,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Result,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Result => Entity::has_one(super::message_result::Entity).into(),
        }
    }
}

impl Related<super::message_result::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Result.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}
