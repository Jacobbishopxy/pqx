//! file: test_cmd.rs
//! author: Jacob Xie
//! date: 2023/05/22 23:15:04 Monday
//! brief:

use std::sync::Arc;

use pqx::ec::util::*;
use pqx::ec::*;
use pqx::error::PqxResult;

// ================================================================================================
// Constants
// ================================================================================================

const CONDA_ENV: &str = "py38";

// ================================================================================================
// mock functions
// ================================================================================================

fn print_stdout(s: String) -> PqxResult<()> {
    println!("print_stdout: {:?}", s);

    Ok(())
}

fn print_stderr(s: String) -> PqxResult<()> {
    println!("print_stderr: {:?}", s);

    Ok(())
}

async fn a_print_stdout(s: String) -> PqxResult<()> {
    println!("print_stdout: {:?}", s);

    Ok(())
}

async fn a_print_stderr(s: String) -> PqxResult<()> {
    println!("print_stderr: {:?}", s);

    Ok(())
}

// ================================================================================================
// Unit tests
// ================================================================================================

#[tokio::test]
async fn cmd_compose_and_exec_success1() {
    let CmdChild { child_stdout, .. } = gen_ping_cmd("github.com").unwrap();

    let (so_tx, so_rx) = gen_execution(1, child_stdout.into(), Arc::new(print_stdout));

    let task = tokio::try_join!(so_tx, so_rx);

    println!("never reach!");
    assert!(task.is_ok());
}

#[tokio::test]
async fn cmd_executor_success1() {
    let mut executor = CmdExecutor::new();
    executor.register_stdout_fn(print_stdout);
    executor.register_stderr_fn(print_stderr);

    let arg = CmdArg::ping("github.com");
    let res = executor.exec(1, arg).await;

    println!("never reach!");
    assert!(res.is_ok());
    println!("{:?}", res.unwrap());
}

#[tokio::test]
async fn cmd_compose_and_exec_success2() {
    let py = cmd_which("python3").unwrap();
    let py = py.strip_suffix("\n").unwrap();
    let dir = join_dir(parent_dir(current_dir().unwrap()).unwrap(), "scripts").unwrap();
    let dir = format!("{}/../scripts", dir.to_str().to_owned().unwrap());
    let cmd = vec!["cd", &dir, "&&", py, "-u", "print_csv_in_line.py"];

    println!("{:?}", py);
    let CmdChild { child_stdout, .. } = gen_bash_cmd(&cmd).unwrap();

    let (so_tx, so_rx) = gen_execution(1, child_stdout.into(), Arc::new(print_stdout));

    let task = tokio::try_join!(so_tx, so_rx);

    assert!(task.is_ok());
}

#[tokio::test]
async fn cmd_executor_success2() {
    let mut executor = CmdExecutor::new();
    executor.register_stdout_fn(&print_stdout);
    executor.register_stderr_fn(&print_stderr);

    let py = cmd_which("python3").unwrap();
    let py = py.strip_suffix("\n").unwrap();
    let dir = join_dir(parent_dir(current_dir().unwrap()).unwrap(), "scripts").unwrap();
    let dir = format!("{}/../scripts", dir.to_str().to_owned().unwrap());
    let cmd = vec!["cd", &dir, "&&", py, "-u", "print_csv_in_line.py"];

    let arg = CmdArg::bash(cmd);
    let res = executor.exec(1, arg).await;

    assert!(res.is_ok());
    println!("{:?}", res.unwrap());
}

#[tokio::test]
async fn cmd_compose_and_exec_success3() {
    let py = cmd_which("python3").unwrap();
    let py = py.strip_suffix("\n").unwrap();
    let dir = join_dir(parent_dir(current_dir().unwrap()).unwrap(), "scripts").unwrap();
    let dir = format!("{}/../scripts", dir.to_str().to_owned().unwrap());
    let cmd = vec!["cd", &dir, "&&", py, "-u", "print_csv_in_line.py"];

    let CmdChild { child_stdout, .. } = gen_bash_cmd(&cmd).unwrap();

    let (so_tx, so_rx) = gen_async_execution(1, child_stdout.into(), Arc::new(a_print_stdout));

    let task = tokio::try_join!(so_tx, so_rx);

    assert!(task.is_ok());
}

#[tokio::test]
async fn cmd_executor_success3() {
    let mut executor = CmdAsyncExecutor::new();
    executor.register_stdout_fn(Arc::new(a_print_stdout));
    executor.register_stderr_fn(Arc::new(a_print_stderr));

    let py = cmd_which("python3").unwrap();
    let py = py.strip_suffix("\n").unwrap();
    let dir = join_dir(parent_dir(current_dir().unwrap()).unwrap(), "scripts").unwrap();
    let dir = format!("{}/../scripts", dir.to_str().to_owned().unwrap());
    let cmd = vec!["cd", &dir, "&&", py, "-u", "print_csv_in_line.py"];

    let arg = CmdArg::bash(cmd);
    let res = executor.exec(1, &arg).await;

    assert!(res.is_ok());
    println!("{:?}", res.unwrap());
}

#[tokio::test]
async fn cmd_compose_and_exec_success4() {
    let dir = join_dir(parent_dir(current_dir().unwrap()).unwrap(), "scripts").unwrap();
    let script = "print_csv_in_line.py";

    let CmdChild { child_stdout, .. } =
        gen_conda_python_cmd(CONDA_ENV, dir.to_str().unwrap(), script).unwrap();

    let (so_tx, so_rx) = gen_async_execution(1, child_stdout.into(), Arc::new(a_print_stdout));

    let task = tokio::try_join!(so_tx, so_rx);

    assert!(task.is_ok());
}

#[tokio::test]
async fn cmd_executor_success4() {
    let mut executor = CmdAsyncExecutor::new();
    executor.register_stdout_fn(Arc::new(a_print_stdout));
    executor.register_stderr_fn(Arc::new(a_print_stderr));

    let dir = join_dir(parent_dir(current_dir().unwrap()).unwrap(), "scripts").unwrap();
    let script = "print_csv_in_line.py".to_string();

    let arg = CmdArg::conda_python(CONDA_ENV, dir.to_string_lossy(), script);
    let res = executor.exec(1, &arg).await;

    assert!(res.is_ok());
    println!("{:?}", res.unwrap());
}
