//! file: message_result.rs
//! author: Jacob Xie
//! date: 2023/06/13 23:05:23 Tuesday
//! brief:

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "message_result")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub exit_code: i16,
    #[sea_orm(nullable)]
    pub result: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::message_history::Entity")]
    History,
}

impl Related<super::message_history::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::History.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}
