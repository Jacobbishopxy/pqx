//! file: util.rs
//! author: Jacob Xie
//! date: 2023/05/26 23:55:37 Friday
//! brief:

use std::{
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
};

use crate::error::PqxResult;

pub fn cmd_which(v: &str) -> PqxResult<String> {
    let output = Command::new("which").arg(v).output()?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn raw_cmd(cmd: &str) -> PqxResult<String> {
    let output = Command::new(cmd).output()?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn cmd_ll(dir: &str) -> PqxResult<Vec<String>> {
    let output = Command::new("ls").current_dir(dir).output()?;

    let s = String::from_utf8_lossy(&output.stdout);

    let res = s.lines().map(String::from).collect::<Vec<_>>();
    Ok(res)
}

pub fn cmd_conda_python(env: &str, dir: &str, script: &str) -> PqxResult<ExitStatus> {
    let es = Command::new("conda")
        .current_dir(dir)
        .arg("run")
        .arg("-n")
        .arg(env)
        .arg("--live-stream")
        .arg("python")
        .arg(script)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    Ok(es)
}

pub fn current_dir() -> PqxResult<PathBuf> {
    let dir = std::env::current_dir()?;

    Ok(dir)
}

pub fn parent_dir(pb: PathBuf) -> PqxResult<PathBuf> {
    let dir = pb.parent().ok_or("parent path fail")?.to_owned();

    Ok(dir)
}

pub fn join_dir<L, R>(lhs: L, rhs: R) -> PqxResult<PathBuf>
where
    L: AsRef<Path>,
    R: AsRef<Path>,
{
    Ok(lhs.as_ref().join(rhs))
}

#[cfg(test)]
mod util_tests {
    use super::*;

    #[test]
    fn cmd_ll_success() {
        let ll = cmd_ll("/");

        assert!(ll.is_ok());
        println!("{:?}", ll.unwrap());
    }

    #[test]
    fn cmd_conda_python_success() {
        let conda_env = "py310";
        let dir = join_dir(parent_dir(current_dir().unwrap()).unwrap(), "scripts").unwrap();
        let script = "print_csv_in_line.py";

        println!("dir: {:?}", dir);

        let cmd = cmd_conda_python(conda_env, dir.to_str().unwrap(), script);

        println!("{:?}", cmd);
    }
}
