//! file: execution.rs
//! author: Jacob Xie
//! date: 2023/06/13 08:50:37 Tuesday
//! brief:

use std::process::ExitStatus;

use async_trait::async_trait;
use pqx::ec::CmdAsyncExecutor;
use pqx::error::{PqxError, PqxResult};
use pqx::mq::{Consumer, ConsumerResult};
use pqx::pqx_util::now;
use tracing::{debug, instrument};

use crate::adt::{Command, ExecutionResult};
use crate::persistence::MessagePersistent;

// ================================================================================================
// Executor
// ================================================================================================

#[derive(Clone)]
pub struct Executor {
    exec: CmdAsyncExecutor,
    persist: MessagePersistent,
}

impl std::fmt::Debug for Executor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Executor")
            .field("exec", &"CmdAsyncExecutor")
            .field("persist", &"MessagePersistent")
            .finish()
    }
}

impl Executor {
    pub fn new(persist: MessagePersistent) -> Self {
        Self {
            exec: CmdAsyncExecutor::new(),
            persist,
        }
    }

    pub fn exec(&self) -> &CmdAsyncExecutor {
        &self.exec
    }

    pub fn exec_mut(&mut self) -> &mut CmdAsyncExecutor {
        &mut self.exec
    }
}

#[async_trait]
impl Consumer<Command, ExitStatus> for Executor {
    #[instrument]
    async fn consume(&mut self, content: &Command) -> PqxResult<ConsumerResult<ExitStatus>> {
        let es = self.exec.exec(1, content.cmd().clone()).await?;
        let res = if es.success() {
            ConsumerResult::success(es)
        } else {
            ConsumerResult::failure(es)
        };
        debug!("{} consumed result: {:?}", now!(), &res);

        Ok(res)
    }

    #[instrument]
    async fn success_callback(&mut self, content: &Command, result: ExitStatus) -> PqxResult<()> {
        let er = ExecutionResult::new(result.code().unwrap_or(0));

        // persist message into db
        let id = self.persist.insert_history(content).await?;
        debug!("{} success insert_history id: {}", now!(), id);
        let id = self.persist.insert_result(id, &er).await?;
        debug!("{} success insert_result id: {}", now!(), id);

        Ok(())
    }

    #[instrument]
    async fn requeue_callback(&mut self, content: &Command, result: ExitStatus) -> PqxResult<()> {
        let ec = result.code().unwrap_or(1);
        let er = ExecutionResult::new_with_result(ec, format!("{:?}", result));

        // persist message into db
        let id = self.persist.insert_history(content).await?;
        debug!("{} requeue insert_history id: {}", now!(), id);
        let id = self.persist.insert_result(id, &er).await?;
        debug!("{} requeue insert_result id: {}", now!(), id);

        Ok(())
    }

    #[instrument]
    async fn discard_callback(&mut self, error: PqxError) -> PqxResult<()> {
        debug!("{} discard error: {:?}", now!(), error);

        Ok(())
    }
}