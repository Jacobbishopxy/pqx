//! file: persistence.rs
//! author: Jacob Xie
//! date: 2023/06/13 11:02:30 Tuesday
//! brief:

use pqx::error::{PqxError, PqxResult};
use pqx::pqx_util::PqxUtilError;
use sea_orm::sea_query::*;
use sea_orm::*;

use crate::adt::{Command, ExecutionResult};
use crate::entities::{message_history, message_result};

// ================================================================================================
// const & types
// ================================================================================================

const MR: &str = "message_result";
const MH: &str = "message_history";

pub type MessageHistoryAndResult = (Command, Option<ExecutionResult>);

// ================================================================================================
// helper
// ================================================================================================

fn gen_check_table_exists_stmt(table_name: &str) -> SelectStatement {
    let cond = Cond::all()
        .add(Expr::col(Alias::new("schemaname")).eq("public"))
        .add(Expr::col(Alias::new("tablename")).eq(table_name));

    let expr = Expr::exists(
        Query::select()
            .from(Alias::new("pg_tables"))
            .cond_where(cond)
            .to_owned(),
    );

    Query::select()
        .expr_as(expr, Alias::new("exists"))
        .to_owned()
}

// ================================================================================================
// MessagePersistent
// ================================================================================================

#[derive(Clone, Debug)]
pub struct MessagePersistent {
    db: DatabaseConnection,
}

impl<'a> MessagePersistent {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn check_existence(&self) -> PqxResult<(bool, bool)> {
        let builder = self.db.get_database_backend();

        let stmt = gen_check_table_exists_stmt(MH);
        let stmt = builder.build(&stmt);

        let res = self
            .db
            .query_one(stmt)
            .await
            .map_err(PqxUtilError::SeaOrm)?
            .ok_or::<PqxError>("query failed".into())?;
        let res1: bool = res.try_get("", "exists").map_err(PqxUtilError::SeaOrm)?;

        let stmt = gen_check_table_exists_stmt(MR);
        let stmt = builder.build(&stmt);

        let res = self
            .db
            .query_one(stmt)
            .await
            .map_err(PqxUtilError::SeaOrm)?
            .ok_or::<PqxError>("query failed".into())?;
        let res2: bool = res.try_get("", "exists").map_err(PqxUtilError::SeaOrm)?;

        Ok((res1, res2))
    }

    pub async fn create_table(&self) {
        let builder = self.db.get_database_backend();
        let schema = Schema::new(builder);

        // create message_history table
        let stmt = builder.build(&schema.create_table_from_entity(message_history::Entity));
        let _ = self.db.execute(stmt).await.map_err(PqxUtilError::SeaOrm);

        // create message_result table
        let stmt = builder.build(&schema.create_table_from_entity(message_result::Entity));
        let _ = self.db.execute(stmt).await.map_err(PqxUtilError::SeaOrm);
    }

    pub async fn drop_table(&self) -> PqxResult<()> {
        let builder = self.db.get_database_backend();

        // drop `message_result`
        let stmt = Table::drop().table(Alias::new(MR)).to_owned();
        let stmt = builder.build(&stmt);
        self.db.execute(stmt).await.map_err(PqxUtilError::SeaOrm)?;

        // drop `message_history`
        let stmt = Table::drop().table(Alias::new(MH)).to_owned();
        let stmt = builder.build(&stmt);
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
