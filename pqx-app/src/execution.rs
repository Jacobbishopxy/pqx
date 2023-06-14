//! file: execution.rs
//! author: Jacob Xie
//! date: 2023/06/13 08:50:37 Tuesday
//! brief:

use std::process::ExitStatus;

use anyhow::Result;
use async_trait::async_trait;
use pqx::ec::CmdAsyncExecutor;
use pqx::error::{PqxError, PqxResult};
use pqx::mq::{Consumer, ConsumerResult};
use pqx::pqx_util::{PersistClient, PersistConn, PqxUtilError};
use sea_orm::ActiveModelTrait;
use tracing::{debug, instrument};

use crate::adt::Command;
use crate::entities::message_history;

// ================================================================================================
// Executor
// ================================================================================================

#[derive(Clone)]
pub struct Executor {
    exec: CmdAsyncExecutor,
    persist: PersistClient,
}

impl std::fmt::Debug for Executor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Executor")
            .field("exec", &"CmdAsyncExecutor")
            .finish()
    }
}

impl Executor {
    pub async fn new(conn: PersistConn) -> Result<Self> {
        let mut persist = PersistClient::new(conn);
        persist.connect().await?;

        Ok(Self {
            exec: CmdAsyncExecutor::new(),
            persist,
        })
    }
}

#[async_trait]
impl Consumer<Command, ExitStatus> for Executor {
    #[instrument]
    async fn consume(&mut self, content: &Command) -> PqxResult<ConsumerResult<ExitStatus>> {
        let res = self.exec.exec(1, content.cmd().clone()).await.map(|es| {
            if es.success() {
                ConsumerResult::success(es)
            } else {
                ConsumerResult::failure(es)
            }
        });
        debug!("{:?}", &res);

        // persist message into db
        let db = self.persist.db().expect("connection established");
        let rcd = message_history::ActiveModel::try_from(content)?;
        rcd.insert(db).await.map_err(PqxUtilError::SeaOrm)?;

        res
    }

    #[instrument]
    async fn success_callback(&mut self, content: &Command, _result: ExitStatus) -> PqxResult<()> {
        //
        Ok(())
    }

    #[instrument]
    async fn requeue_callback(&mut self, content: &Command, _result: ExitStatus) -> PqxResult<()> {
        //
        Ok(())
    }

    #[instrument]
    async fn discard_callback(&mut self, _error: PqxError) -> PqxResult<()> {
        //
        Ok(())
    }
}
