//! file: execution.rs
//! author: Jacob Xie
//! date: 2023/06/13 08:50:37 Tuesday
//! brief:

use async_trait::async_trait;
use pqx::ec::CmdAsyncExecutor;
use pqx::error::{PqxError, PqxResult};
use pqx::mq::Consumer;
use tracing::instrument;

use crate::adt::Command;

// ================================================================================================
// Executor
// ================================================================================================

#[derive(Clone)]
pub struct Executor {
    exec: CmdAsyncExecutor,
    // persist:
}

impl std::fmt::Debug for Executor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Executor")
            .field("exec", &"CmdAsyncExecutor")
            .finish()
    }
}

impl Executor {
    pub fn new() -> Self {
        Self {
            exec: CmdAsyncExecutor::new(),
        }
    }
}

#[async_trait]
impl Consumer<Command> for Executor {
    #[instrument]
    async fn consume(&mut self, content: &Command) -> PqxResult<bool> {
        let es = self.exec.exec(1, content.cmd().clone()).await?;

        // TODO: persist

        Ok(es.success())
    }

    #[instrument]
    async fn success_callback(&mut self, content: &Command) -> PqxResult<()> {
        //
        Ok(())
    }

    #[instrument]
    async fn requeue_callback(&mut self, content: &Command) -> PqxResult<()> {
        //
        Ok(())
    }

    #[instrument]
    async fn discard_callback(&mut self, _error: PqxError) -> PqxResult<()> {
        //
        Ok(())
    }
}
