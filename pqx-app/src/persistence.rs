//! file: persistence.rs
//! author: Jacob Xie
//! date: 2023/06/13 11:02:30 Tuesday
//! brief:

use pqx::error::PqxResult;
use pqx::pqx_util::PqxUtilError;
use sea_orm::{DatabaseConnection, EntityTrait};

use crate::adt::{Command, ExecutionResult};
use crate::entities::{message_history, message_result};

// ================================================================================================
// MessagePersistent
// ================================================================================================

#[derive(Clone)]
pub struct MessagePersistent<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> MessagePersistent<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn insert_history(&self, cmd: &Command) -> PqxResult<i64> {
        let am = message_history::ActiveModel::try_from(cmd)?;
        let id = message_history::Entity::insert(am)
            .exec(self.db)
            .await
            .map_err(PqxUtilError::SeaOrm)?
            .last_insert_id;

        Ok(id)
    }

    pub async fn insert_result(&self, history_id: i64, res: &ExecutionResult) -> PqxResult<i64> {
        let am = res.into_active_model(history_id);
        let id = message_result::Entity::insert(am)
            .exec(self.db)
            .await
            .map_err(PqxUtilError::SeaOrm)?
            .last_insert_id;

        Ok(id)
    }

    // TODO: query, pagination, sort, find...
}
