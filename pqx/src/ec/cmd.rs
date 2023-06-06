//! file: cmd.rs
//! author: Jacob Xie
//! date: 2023/05/22 20:49:10 Monday
//! brief:

use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::io::{BufRead, BufReader, Read};
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};

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

pub(crate) async fn send_child_std<R: Read>(r: R, tx: Sender<String>) -> PqxResult<()> {
    for line in BufReader::new(r).lines() {
        let line = line?;

        tx.send(line)
            .await
            .map_err(|_| PqxError::custom("send_child_std async send"))?;
    }

    Ok(())
}

impl ChildStdPipe {
    pub(crate) async fn send_child_std(self, tx: Sender<String>) -> PqxResult<()> {
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

pub fn gen_bash_cmd<I, S>(cmd: I) -> PqxResult<CmdChild>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut child = Command::new("bash")
        .arg("-c")
        .args(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    Ok(CmdChild::new(child, child_stdout, child_stderr))
}

pub fn gen_ssh_cmd<I, S>(ip: &str, user: &str, cmd: I) -> PqxResult<CmdChild>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut child = Command::new("ssh")
        .arg(format!("{}@{}", user, ip))
        .args(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    Ok(CmdChild::new(child, child_stdout, child_stderr))
}

pub fn gen_sshpass_cmd<I, S>(ip: &str, user: &str, pass: &str, cmd: I) -> PqxResult<CmdChild>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut child = Command::new("sshpass")
        .arg("-p")
        .arg(pass)
        .arg("ssh")
        .arg(format!("{}@{}", user, ip))
        .args(cmd)
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
        .arg("-u")
        .arg(script)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    Ok(CmdChild::new(child, child_stdout, child_stderr))
}

pub fn gen_docker_exec_cmd<I, S>(container: &str, cmd: I) -> PqxResult<CmdChild>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut child = Command::new("docker")
        .arg("exec")
        .arg(container)
        .args(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    Ok(CmdChild::new(child, child_stdout, child_stderr))
}

// ================================================================================================
// CmdArg
// ================================================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CmdArg {
    Ping {
        addr: String,
    },
    Bash {
        cmd: Vec<String>,
    },
    Ssh {
        ip: String,
        user: String,
        cmd: Vec<String>,
    },
    Sshpass {
        ip: String,
        user: String,
        pass: String,
        cmd: Vec<String>,
    },
    CondaPython {
        env: String,
        dir: String,
        script: String,
    },
    DockerExec {
        container: String,
        cmd: Vec<String>,
    },
}

impl CmdArg {
    pub fn ping(addr: impl Into<String>) -> Self {
        Self::Ping { addr: addr.into() }
    }

    pub fn bash<I, S>(cmd: I) -> Self
    where
        I: IntoIterator<Item = S>,
        String: From<S>,
    {
        Self::Bash {
            cmd: cmd.into_iter().map(String::from).collect(),
        }
    }

    pub fn ssh<I, S>(ip: impl Into<String>, user: impl Into<String>, cmd: I) -> Self
    where
        I: IntoIterator<Item = S>,
        String: From<S>,
    {
        Self::Ssh {
            ip: ip.into(),
            user: user.into(),
            cmd: cmd.into_iter().map(String::from).collect(),
        }
    }

    pub fn sshpass<I, S>(
        ip: impl Into<String>,
        user: impl Into<String>,
        pass: impl Into<String>,
        cmd: I,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        String: From<S>,
    {
        Self::Sshpass {
            ip: ip.into(),
            user: user.into(),
            pass: pass.into(),
            cmd: cmd.into_iter().map(String::from).collect(),
        }
    }

    pub fn conda_python(
        env: impl Into<String>,
        dir: impl Into<String>,
        script: impl Into<String>,
    ) -> Self {
        Self::CondaPython {
            env: env.into(),
            dir: dir.into(),
            script: script.into(),
        }
    }

    pub fn docker_exec<I, S>(container: impl Into<String>, cmd: I) -> Self
    where
        I: IntoIterator<Item = S>,
        String: From<S>,
    {
        Self::DockerExec {
            container: container.into(),
            cmd: cmd.into_iter().map(String::from).collect(),
        }
    }

    pub fn gen_cmd(&self) -> PqxResult<CmdChild> {
        match self {
            CmdArg::Ping { addr } => gen_ping_cmd(addr),
            CmdArg::Bash { cmd } => gen_bash_cmd(cmd),
            CmdArg::Ssh { ip, cmd, user } => gen_ssh_cmd(ip, user, cmd),
            CmdArg::Sshpass {
                ip,
                user,
                pass,
                cmd,
            } => gen_sshpass_cmd(ip, user, pass, cmd),
            CmdArg::CondaPython { env, dir, script } => gen_conda_python_cmd(env, dir, script),
            CmdArg::DockerExec { container, cmd } => gen_docker_exec_cmd(container, cmd),
        }
    }
}
