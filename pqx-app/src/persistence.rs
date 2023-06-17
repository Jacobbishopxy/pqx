//! file: persistence.rs
//! author: Jacob Xie
//! date: 2023/06/13 11:02:30 Tuesday
//! brief:

use pqx::error::{PqxError, PqxResult};
use pqx::pqx_util::PqxUtilError;
use sea_orm::*;

use crate::adt::{Command, ExecutionResult};
use crate::entities::{message_history, message_result};

// ================================================================================================
// types
// ================================================================================================

pub type MessageHistoryAndResult = (Command, Option<ExecutionResult>);

// ================================================================================================
// MessagePersistent
// ================================================================================================

#[derive(Clone)]
pub struct MessagePersistent {
    db: DatabaseConnection,
}

impl<'a> MessagePersistent {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create_table(&self) -> PqxResult<()> {
        let builder = self.db.get_database_backend();
        let schema = Schema::new(builder);

        // create message_history table
        let stmt = builder.build(&schema.create_table_from_entity(message_history::Entity));
        self.db.execute(stmt).await.map_err(PqxUtilError::SeaOrm)?;

        // create message_result table
        let stmt = builder.build(&schema.create_table_from_entity(message_result::Entity));
        self.db.execute(stmt).await.map_err(PqxUtilError::SeaOrm)?;

        Ok(())
    }

    pub async fn insert_history(&self, cmd: &Command) -> PqxResult<i64> {
        let am = message_history::ActiveModel::try_from(cmd)?;
        let id = message_history::Entity::insert(am)
            .exec(&self.db)
            .await
            .map_err(PqxUtilError::SeaOrm)?
            .last_insert_id;

        Ok(id)
    }

    pub async fn insert_result(&self, history_id: i64, res: &ExecutionResult) -> PqxResult<i64> {
        let am = res.into_active_model(history_id);
        let id = message_result::Entity::insert(am)
            .exec(&self.db)
            .await
            .map_err(PqxUtilError::SeaOrm)?
            .last_insert_id;

        Ok(id)
    }

    pub async fn find_one(&self, history_id: i64) -> PqxResult<MessageHistoryAndResult> {
        message_history::Entity::find_by_id(history_id)
            .find_also_related(message_result::Entity)
            .one(&self.db)
            .await
            .map_err(PqxUtilError::SeaOrm)?
            .map(|(c, r)| {
                let cmd = Command::try_from(c)?;
                let res = r.map(ExecutionResult::from);
                Ok::<MessageHistoryAndResult, PqxError>((cmd, res))
            })
            .transpose()?
            .ok_or("history not found".into())
    }

    pub async fn find_many(
        &self,
        history_ids: impl IntoIterator<Item = i64>,
    ) -> PqxResult<Vec<MessageHistoryAndResult>> {
        message_history::Entity::find()
            .filter(message_history::Column::Id.is_in(history_ids))
            .find_also_related(message_result::Entity)
            .all(&self.db)
            .await
            .map_err(PqxUtilError::SeaOrm)?
            .into_iter()
            .map(|(c, r)| {
                let cmd = Command::try_from(c)?;
                let res = r.map(ExecutionResult::from);
                Ok((cmd, res))
            })
            .collect::<PqxResult<Vec<_>>>()
    }

    pub async fn find_by_pagination(
        &self,
        page: u64,
        page_size: u64,
    ) -> PqxResult<Vec<MessageHistoryAndResult>> {
        message_history::Entity::find()
            .find_also_related(message_result::Entity)
            .paginate(&self.db, page_size)
            .fetch_page(page)
            .await
            .map_err(PqxUtilError::SeaOrm)?
            .into_iter()
            .map(|(c, r)| {
                let cmd = Command::try_from(c)?;
                let res = r.map(ExecutionResult::from);
                Ok((cmd, res))
            })
            .collect::<PqxResult<Vec<_>>>()
    }
}
