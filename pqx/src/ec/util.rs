//! file: util.rs
//! author: Jacob Xie
//! date: 2023/05/26 23:55:37 Friday
//! brief:

use std::process::Command;

use crate::error::PqxResult;

pub fn cmd_which(v: &str) -> PqxResult<String> {
    let output = Command::new("which").arg(v).output()?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn raw_cmd(cmd: &str) -> PqxResult<String> {
    let output = Command::new(cmd).output()?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
