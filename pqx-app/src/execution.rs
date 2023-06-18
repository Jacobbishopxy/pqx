//! file: execution.rs
//! author: Jacob Xie
//! date: 2023/06/13 08:50:37 Tuesday
//! brief:

use std::process::ExitStatus;

use async_trait::async_trait;
use pqx::ec::CmdAsyncExecutor;
use pqx::error::{PqxError, PqxResult};
use pqx::mq::{Consumer, ConsumerResult, Retry};
use pqx::pqx_util::now;
use tracing::{debug, instrument};

use crate::adt::{Command, ExecutionResult};
use crate::persistence::MessagePersistent;

// ================================================================================================
// Executor
// ================================================================================================

#[derive(Clone)]
pub struct Executor {
    delayed_exchange: String,
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
    pub fn new(delayed_exchange: impl Into<String>, persist: MessagePersistent) -> Self {
        Self {
            delayed_exchange: delayed_exchange.into(),
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
        let es = self.exec.exec(1, content.cmd()).await?;
        let res = if es.success() {
            ConsumerResult::success(es)
        } else {
            // instead of `Requeue`, use `Retry`
            ConsumerResult::retry(es)
        };
        debug!("{} consumed result: {:?}", now!(), &res);

        Ok(res)
    }

    fn gen_retry(&self, message: &Command) -> Retry {
        Retry {
            exchange: self.delayed_exchange.clone(),
            routing_key: String::from(""),
            poke: message.poke.unwrap_or(10000),
            retries: message.retry.unwrap_or(1),
        }
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
    async fn retry_callback(&mut self, content: &Command, result: ExitStatus) -> PqxResult<()> {
        let ec = result.code().unwrap_or(1);
        let er = ExecutionResult::new_with_result(ec, format!("{:?}", result));

        // persist message into db
        let id = self.persist.insert_history(content).await?;
        debug!("{} retry insert_history id: {}", now!(), id);
        let id = self.persist.insert_result(id, &er).await?;
        debug!("{} retry insert_result id: {}", now!(), id);

        Ok(())
    }

    #[instrument]
    async fn discard_callback(&mut self, error: PqxError) -> PqxResult<()> {
        debug!("{} discard error: {:?}", now!(), error);

        Ok(())
    }
}
