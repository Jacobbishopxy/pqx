//! file: message_result.rs
//! author: Jacob Xie
//! date: 2023/06/13 23:05:23 Tuesday
//! brief:

use sea_orm::entity::prelude::*;

// ================================================================================================
// Model: message_result
// ================================================================================================

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "message_result")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub history_id: i64,
    pub exit_code: i16,
    #[sea_orm(nullable)]
    pub result: Option<String>,
    pub time: chrono::DateTime<chrono::Local>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    History,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::History => Entity::belongs_to(super::message_history::Entity)
                .from(Column::HistoryId)
                .to(super::message_history::Column::Id)
                .into(),
        }
    }
}

impl Related<super::message_history::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::History.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}
