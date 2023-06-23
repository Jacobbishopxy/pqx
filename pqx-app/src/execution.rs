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

#[derive(Clone, Debug)]
pub struct Executor {
    delayed_exchange: String,
    exec: CmdAsyncExecutor,
    persist: MessagePersistent,
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
            ConsumerResult::retry(Some(es))
        };
        debug!("{} consumed result: {:?}", now!(), &res);

        Ok(res)
    }

    fn gen_retry(&self, message: &Command) -> Retry {
        Retry::new(
            &self.delayed_exchange,
            "",                         // header exchange, no need routing_key
            message.poke.unwrap_or(10), // default 10s
            message.retry.unwrap_or(1), // default retry once
        )
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
    async fn retry_callback(
        &mut self,
        content: &Command,
        result: Option<ExitStatus>,
    ) -> PqxResult<()> {
        // persist message into db
        let id = self.persist.insert_history(content).await?;
        debug!("{} retry insert_history id: {}", now!(), id);

        let er = match result {
            Some(es) => {
                let ec = es.code().unwrap_or(1);
                ExecutionResult::new_with_result(ec, format!("{:?}", result))
            }
            None => ExecutionResult::new_with_result(1, "timeout"),
        };
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
