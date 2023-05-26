//! file: cmd.rs
//! author: Jacob Xie
//! date: 2023/05/22 20:49:10 Monday
//! brief:

use futures::future::{BoxFuture, Future};
use std::io::{BufRead, BufReader, Read};
use std::process::{Child, ChildStderr, ChildStdout, Command, ExitStatus, Stdio};

use tokio::sync::mpsc::Sender;

use crate::error::{PqxError, PqxResult};

// ================================================================================================
// ChildStdPipe
// ================================================================================================

pub enum ChildStdPipe {
    Out(ChildStdout),
    Err(ChildStderr),
}

impl From<ChildStdout> for ChildStdPipe {
    fn from(value: ChildStdout) -> Self {
        ChildStdPipe::Out(value)
    }
}

impl From<ChildStderr> for ChildStdPipe {
    fn from(value: ChildStderr) -> Self {
        ChildStdPipe::Err(value)
    }
}

async fn send_child_std<R: Read>(r: R, tx: Sender<String>) -> PqxResult<()> {
    for line in BufReader::new(r).lines() {
        let line = line?;
        // println!("send_child_std > {:?}", line);

        tx.send(line)
            .await
            .map_err(|_| PqxError::custom("send_child_std async send"))?;
    }

    Ok(())
}

impl ChildStdPipe {
    async fn send_child_std(self, tx: Sender<String>) -> PqxResult<()> {
        match self {
            ChildStdPipe::Out(o) => send_child_std(o, tx).await,
            ChildStdPipe::Err(e) => send_child_std(e, tx).await,
        }
    }
}

// ================================================================================================
// commands generators
// ================================================================================================

pub struct CmdChild {
    pub child: Child,
    pub child_stdout: ChildStdout,
    pub child_stderr: ChildStderr,
}

impl CmdChild {
    pub fn new(child: Child, child_std_out: ChildStdout, child_std_err: ChildStderr) -> Self {
        Self {
            child,
            child_stdout: child_std_out,
            child_stderr: child_std_err,
        }
    }
}

pub fn gen_ping_cmd(addr: &str) -> PqxResult<CmdChild> {
    let mut child = Command::new("ping")
        .arg(addr)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    Ok(CmdChild::new(child, child_stdout, child_stderr))
}

pub fn gen_bash_cmd(cmd: &str) -> PqxResult<CmdChild> {
    let mut child = Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    Ok(CmdChild::new(child, child_stdout, child_stderr))
}

pub fn gen_ssh_cmd(ip: &str, cmd: &str, user: Option<&str>) -> PqxResult<CmdChild> {
    // ssh -t {ip} -o StrictHostKeyChecking=no '{cmd}'
    // ssh -t {ip} -o StrictHostKeyChecking=no 'sudo -u {user} -H sh -c "cd ~; . ~/.bashrc; {cmd}"'

    let mut command = Command::new("ssh");
    command
        .arg("-t")
        .arg(ip)
        .arg("-o")
        .arg("StrictHostKeyChecking=no");

    if let Some(usr) = user {
        command.arg(format!("sudo -u {} -H sh -c", usr)).arg(cmd);
    }

    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    Ok(CmdChild::new(child, child_stdout, child_stderr))
}

pub fn gen_conda_python_cmd(env: &str, dir: &str, script: &str) -> PqxResult<CmdChild> {
    let mut child = Command::new("conda")
        .current_dir(dir)
        .arg("run")
        .arg("-n")
        .arg(env)
        .arg("--live-stream")
        .arg("python")
        .arg(script)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    Ok(CmdChild::new(child, child_stdout, child_stderr))
}

pub enum CmdArg<'a> {
    Ping {
        addr: &'a str,
    },
    Bash {
        cmd: &'a str,
    },
    Ssh {
        ip: &'a str,
        cmd: &'a str,
        user: Option<&'a str>,
    },
    CondaPython {
        env: &'a str,
        dir: &'a str,
        script: &'a str,
    },
}

impl<'a> CmdArg<'a> {
    pub fn gen_cmd(&self) -> PqxResult<CmdChild> {
        match self {
            CmdArg::Ping { addr } => gen_ping_cmd(addr),
            CmdArg::Bash { cmd } => gen_bash_cmd(cmd),
            CmdArg::Ssh { ip, cmd, user } => gen_ssh_cmd(ip, cmd, *user),
            CmdArg::CondaPython { env, dir, script } => gen_conda_python_cmd(env, dir, script),
        }
    }
}

// ================================================================================================
// CmdExecutor
// ================================================================================================

pub type SyncFn = &'static dyn Fn(String) -> PqxResult<()>;

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

    pub fn register_stdout_fn(&mut self, f: SyncFn) -> &mut Self {
        self.stdout_fn = Some(f);

        self
    }

    pub fn register_stderr_fn(&mut self, f: SyncFn) -> &mut Self {
        self.stderr_fn = Some(f);

        self
    }

    pub async fn exec(&self, channel_buffer: usize, arg: CmdArg<'_>) -> PqxResult<ExitStatus> {
        let CmdChild {
            mut child,
            child_stdout,
            child_stderr,
        } = arg.gen_cmd()?;

        exec_cmd(
            channel_buffer,
            self.stdout_fn.as_ref().map(|f| (child_stdout, *f)),
            self.stderr_fn.as_ref().map(|f| (child_stderr, *f)),
        )
        .await?;

        Ok(child.wait()?)
    }
}

// ================================================================================================
// CmdAsyncExecutor
// ================================================================================================

pub trait AsyncFn {
    fn call(&self, input: String) -> BoxFuture<'static, PqxResult<()>>;
}

impl<T, F> AsyncFn for T
where
    T: Fn(String) -> F,
    F: Future<Output = PqxResult<()>> + Send + 'static,
{
    fn call(&self, input: String) -> BoxFuture<'static, PqxResult<()>> {
        Box::pin(self(input))
    }
}

pub fn gen_async_execution<'a>(
    channel_buffer: usize,
    pipe: ChildStdPipe,
    f: &'static dyn AsyncFn,
) -> (
    impl Future<Output = Result<(), &'_ str>>,
    impl Future<Output = Result<(), &'_ str>>,
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
    fo: Option<(ChildStdout, &'static dyn AsyncFn)>,
    fe: Option<(ChildStderr, &'static dyn AsyncFn)>,
) -> PqxResult<()> {
    let chn_o = fo.map(|(o, f)| gen_async_execution(channel_buffer, o.into(), f));
    let chn_e = fe.map(|(e, f)| gen_async_execution(channel_buffer, e.into(), f));

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

pub struct CmdAsyncExecutor {
    stdout_fn: Option<&'static dyn AsyncFn>,
    stderr_fn: Option<&'static dyn AsyncFn>,
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

    pub fn register_stdout_fn(&mut self, f: &'static dyn AsyncFn) -> &mut Self {
        self.stdout_fn = Some(f);

        self
    }

    pub fn register_stderr_fn(&mut self, f: &'static dyn AsyncFn) -> &mut Self {
        self.stderr_fn = Some(f);

        self
    }

    pub async fn exec(&self, channel_buffer: usize, arg: CmdArg<'_>) -> PqxResult<ExitStatus> {
        let CmdChild {
            mut child,
            child_stdout,
            child_stderr,
        } = arg.gen_cmd()?;

        exec_async_cmd(
            channel_buffer,
            self.stdout_fn.as_ref().map(|f| (child_stdout, *f)),
            self.stderr_fn.as_ref().map(|f| (child_stderr, *f)),
        )
        .await?;

        Ok(child.wait()?)
    }
}
