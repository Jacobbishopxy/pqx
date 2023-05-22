//! file: cmd.rs
//! author: Jacob Xie
//! date: 2023/05/22 20:49:10 Monday
//! brief:

use std::io::{BufRead, BufReader};
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};

use tokio::sync::mpsc::Sender;

use crate::error::{PqxError, PqxResult};

async fn stdout_lines(stdout: ChildStdout, tx: Sender<String>) -> PqxResult<()> {
    for line in BufReader::new(stdout).lines() {
        let line = line?;

        // println!("stdout > {:?}", line);

        tx.send(line)
            .await
            .map_err(|_| PqxError::custom("stdout_lines async send"))?;
    }

    Ok(())
}

async fn stderr_lines(stderr: ChildStderr, tx: Sender<String>) -> PqxResult<()> {
    for line in BufReader::new(stderr).lines() {
        let line = line?;

        // println!("stderr > {:?}", line);

        tx.send(line)
            .await
            .map_err(|_| PqxError::custom("stderr_lines async send"))?;
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

pub async fn async_cmd<F>(
    channel_buffer: usize,
    fo: Option<(ChildStdout, F)>,
    fe: Option<(ChildStderr, F)>,
) -> PqxResult<()>
where
    F: Fn(String) + Send + 'static,
{
    let chn_o = fo.map(|(child_stdout, f)| {
        let (stdout_tx, mut stdout_rx) = tokio::sync::mpsc::channel(channel_buffer);

        let so_tx = async move {
            stdout_lines(child_stdout, stdout_tx)
                .await
                .map_err(|_| "so_tx fail")
        };

        let so_rx = async move {
            while let Some(msg) = stdout_rx.recv().await {
                f(msg);
            }
            Ok(())
        };

        (so_tx, so_rx)
    });

    let chn_e = fe.map(|(child_stderr, f)| {
        let (stderr_tx, mut stderr_rx) = tokio::sync::mpsc::channel(channel_buffer);

        let se_tx = async move {
            stderr_lines(child_stderr, stderr_tx)
                .await
                .map_err(|_| "se_tx fail")
        };

        let se_rx = async move {
            while let Some(msg) = stderr_rx.recv().await {
                f(msg);
            }
            Ok(())
        };

        (se_tx, se_rx)
    });

    match (chn_o, chn_e) {
        (None, None) => {}
        (None, Some((se_tx, se_rx))) => {
            tokio::try_join!(se_tx, se_rx)?;
        }
        (Some((so_tx, so_rx)), None) => {
            tokio::try_join!(so_tx, so_rx)?;
        }
        (Some((so_tx, so_rx)), Some((se_tx, se_rx))) => {
            tokio::try_join!(se_tx, se_rx, so_tx, so_rx)?;
        }
    };

    Ok(())
}
