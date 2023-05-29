//! file: cmd.rs
//! author: Jacob Xie
//! date: 2023/05/22 20:49:10 Monday
//! brief:

use serde::{Deserialize, Serialize};
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
        // println!("send_child_std > {:?}", line);

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
        .arg("-u")
        .arg(script)
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

#[derive(Debug, Serialize, Deserialize)]
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
