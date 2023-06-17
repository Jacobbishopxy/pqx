//! file: exec.rs
//! author: Jacob Xie
//! date: 2023/05/29 08:30:42 Monday
//! brief:

use futures::future::{BoxFuture, Future};
use std::process::{ChildStderr, ChildStdout, ExitStatus};
use std::sync::Arc;

use crate::error::PqxResult;

use super::{send_child_std, ChildStdPipe, CmdArg, CmdChild};

// ================================================================================================
// CmdExecutor
// ================================================================================================

pub type SyncFn = Arc<dyn Fn(String) -> PqxResult<()>>;

pub fn gen_execution<'a>(
    channel_buffer: usize,
    pipe: ChildStdPipe,
    f: SyncFn,
) -> (
    impl Future<Output = Result<(), &'a str>>,
    impl Future<Output = Result<(), &'a str>>,
) {
    let (stdout_tx, mut stdout_rx) = tokio::sync::mpsc::channel(channel_buffer);

    let so_tx = async move {
        match pipe {
            ChildStdPipe::Out(o) => send_child_std(o, stdout_tx).await.map_err(|_| "so_tx fail"),
            ChildStdPipe::Err(e) => send_child_std(e, stdout_tx).await.map_err(|_| "se_tx fail"),
        }
    };

    let so_rx = async move {
        while let Some(msg) = stdout_rx.recv().await {
            f(msg).map_err(|_| "so_rx fail")?;
        }
        Ok(())
    };

    (so_tx, so_rx)
}

async fn exec_cmd(
    channel_buffer: usize,
    fo: Option<(ChildStdout, SyncFn)>,
    fe: Option<(ChildStderr, SyncFn)>,
) -> PqxResult<()> {
    let chn_o = fo.map(|(o, f)| gen_execution(channel_buffer, o.into(), f));
    let chn_e = fe.map(|(e, f)| gen_execution(channel_buffer, e.into(), f));

    match (chn_o, chn_e) {
        (None, None) => {}
        (None, Some((se_tx, se_rx))) => {
            tokio::try_join!(se_tx, se_rx)?;
        }
        (Some((so_tx, so_rx)), None) => {
            tokio::try_join!(so_tx, so_rx)?;
        }
        (Some((so_tx, so_rx)), Some((se_tx, se_rx))) => {
            tokio::try_join!(so_tx, so_rx)?;
            tokio::try_join!(se_tx, se_rx)?;
        }
    };

    Ok(())
}

#[derive(Clone)]
pub struct CmdExecutor {
    stdout_fn: Option<SyncFn>,
    stderr_fn: Option<SyncFn>,
}

impl CmdExecutor {
    pub fn new() -> Self {
        Self {
            stdout_fn: None,
            stderr_fn: None,
        }
    }

    pub fn has_stdout_fn(&self) -> bool {
        self.stdout_fn.is_some()
    }

    pub fn has_stderr_fn(&self) -> bool {
        self.stderr_fn.is_some()
    }

    pub fn register_stdout_fn(
        &mut self,
        f: impl Fn(String) -> PqxResult<()> + 'static,
    ) -> &mut Self {
        self.stdout_fn = Some(Arc::new(f));

        self
    }

    pub fn register_stderr_fn(
        &mut self,
        f: impl Fn(String) -> PqxResult<()> + 'static,
    ) -> &mut Self {
        self.stderr_fn = Some(Arc::new(f));

        self
    }

    pub async fn exec(&self, channel_buffer: usize, arg: CmdArg) -> PqxResult<ExitStatus> {
        let CmdChild {
            mut child,
            child_stdout,
            child_stderr,
        } = arg.gen_cmd()?;

        exec_cmd(
            channel_buffer,
            self.stdout_fn.as_ref().map(|f| (child_stdout, f.clone())),
            self.stderr_fn.as_ref().map(|f| (child_stderr, f.clone())),
        )
        .await?;

        Ok(child.wait()?)
    }
}

// ================================================================================================
// CmdAsyncExecutor
// ================================================================================================

pub trait AsyncFn: Send + Sync {
    fn call<'a>(&'a self, input: String) -> BoxFuture<'a, PqxResult<()>>;
}

impl<T, F> AsyncFn for T
where
    T: Fn(String) -> F,
    T: Send + Sync,
    F: Future<Output = PqxResult<()>> + Send + 'static,
{
    fn call<'a>(&'a self, input: String) -> BoxFuture<'a, PqxResult<()>> {
        Box::pin(self(input))
    }
}

pub fn gen_async_execution<'a>(
    channel_buffer: usize,
    pipe: ChildStdPipe,
    f: Arc<dyn AsyncFn>,
) -> (
    impl Future<Output = Result<(), &'a str>>,
    impl Future<Output = Result<(), &'a str>>,
) {
    let (std_tx, mut std_rx) = tokio::sync::mpsc::channel(channel_buffer);

    let so_tx = async move {
        pipe.send_child_std(std_tx)
            .await
            .map_err(|_| "so_tx fail")?;

        Ok(())
    };

    let so_rx = async move {
        // if consuming speed (`f(msg).await`) is less than recving speed (`std_rx`), then the `msg` buffer will discard
        // new `msg` until channel has space again. check `tokio::sync::mpsc::channel`
        while let Some(msg) = std_rx.recv().await {
            f.call(msg).await.map_err(|_| "so_rx fail")?;
        }

        Ok(())
    };

    (so_tx, so_rx)
}

async fn exec_async_cmd<'a>(
    channel_buffer: usize,
    fo: Option<(ChildStdout, Arc<dyn AsyncFn>)>,
    fe: Option<(ChildStderr, Arc<dyn AsyncFn>)>,
) -> PqxResult<()> {
    let chn_o = fo.map(|(o, f)| gen_async_execution(channel_buffer, o.into(), f.clone()));
    let chn_e = fe.map(|(e, f)| gen_async_execution(channel_buffer, e.into(), f.clone()));

    match (chn_o, chn_e) {
        (None, None) => {}
        (None, Some((se_tx, se_rx))) => {
            tokio::try_join!(se_tx, se_rx)?;
        }
        (Some((so_tx, so_rx)), None) => {
            tokio::try_join!(so_tx, so_rx)?;
        }
        (Some((so_tx, so_rx)), Some((se_tx, se_rx))) => {
            tokio::try_join!(so_tx, so_rx)?;
            tokio::try_join!(se_tx, se_rx)?;
        }
    };

    Ok(())
}

#[derive(Clone)]
pub struct CmdAsyncExecutor {
    stdout_fn: Option<Arc<dyn AsyncFn>>,
    stderr_fn: Option<Arc<dyn AsyncFn>>,
}

impl CmdAsyncExecutor {
    pub fn new() -> Self {
        Self {
            stdout_fn: None,
            stderr_fn: None,
        }
    }

    pub fn has_stdout_fn(&self) -> bool {
        self.stdout_fn.is_some()
    }

    pub fn has_stderr_fn(&self) -> bool {
        self.stderr_fn.is_some()
    }

    pub fn register_stdout_fn(&mut self, f: Arc<dyn AsyncFn>) -> &mut Self {
        self.stdout_fn = Some(f);

        self
    }

    pub fn register_stderr_fn(&mut self, f: Arc<dyn AsyncFn>) -> &mut Self {
        self.stderr_fn = Some(f);

        self
    }

    pub async fn exec(&self, channel_buffer: usize, arg: &CmdArg) -> PqxResult<ExitStatus> {
        let CmdChild {
            mut child,
            child_stdout,
            child_stderr,
        } = arg.gen_cmd()?;

        exec_async_cmd(
            channel_buffer,
            self.stdout_fn.as_ref().map(|f| (child_stdout, f.clone())),
            self.stderr_fn.as_ref().map(|f| (child_stderr, f.clone())),
        )
        .await?;

        Ok(child.wait()?)
    }
}
