//! file: cmd.rs
//! author: Jacob Xie
//! date: 2023/05/22 20:49:10 Monday
//! brief:

use std::future::Future;
use std::io::{BufRead, BufReader, Read};
use std::process::{Child, ChildStderr, ChildStdout, Command, ExitStatus, Stdio};

use tokio::sync::mpsc::Sender;

use crate::error::{PqxError, PqxResult};

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

pub fn gen_ping_cmd(addr: &str) -> PqxResult<(Child, ChildStdout, ChildStderr)> {
    let mut child = Command::new("ping")
        .arg(addr)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    Ok((child, child_stdout, child_stderr))
}

pub fn gen_bash_cmd(cmd: &str) -> PqxResult<(Child, ChildStdout, ChildStderr)> {
    let mut child = Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    Ok((child, child_stdout, child_stderr))
}

pub fn gen_ssh_cmd(
    ip: &str,
    cmd: &str,
    user: Option<&str>,
) -> PqxResult<(Child, ChildStdout, ChildStderr)> {
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

    Ok((child, child_stdout, child_stderr))
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
}

pub fn gen_async_execution<'a, F: Fn(String)>(
    channel_buffer: usize,
    pipe: ChildStdPipe,
    f: F,
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
            f(msg);
        }
        Ok(())
    };

    (so_tx, so_rx)
}

async fn exec_async_cmd<FO, FE>(
    channel_buffer: usize,
    fo: Option<(ChildStdout, FO)>,
    fe: Option<(ChildStderr, FE)>,
) -> PqxResult<()>
where
    FO: Fn(String),
    FE: Fn(String),
{
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

pub struct CmdExecutor {
    stdout_fn: Option<Box<dyn Fn(String)>>,
    stderr_fn: Option<Box<dyn Fn(String)>>,
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

    pub fn register_stdout_fn<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(String) + 'static,
    {
        self.stdout_fn = Some(Box::new(f));

        self
    }

    pub fn register_stderr_fn<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(String) + 'static,
    {
        self.stderr_fn = Some(Box::new(f));

        self
    }

    pub async fn exec(&self, channel_buffer: usize, arg: CmdArg<'_>) -> PqxResult<ExitStatus> {
        let (mut child, child_stdout, child_stderr) = match arg {
            CmdArg::Ping { addr } => gen_ping_cmd(addr)?,
            CmdArg::Bash { cmd } => gen_bash_cmd(cmd)?,
            CmdArg::Ssh { ip, cmd, user } => gen_ssh_cmd(ip, cmd, user)?,
        };

        exec_async_cmd(
            channel_buffer,
            self.stdout_fn.as_ref().map(|f| (child_stdout, f)),
            self.stderr_fn.as_ref().map(|f| (child_stderr, f)),
        )
        .await?;

        Ok(child.wait()?)
    }
}
