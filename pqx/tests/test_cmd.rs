//! file: test_cmd.rs
//! author: Jacob Xie
//! date: 2023/05/22 23:15:04 Monday
//! brief:

use std::sync::Arc;

use pqx::ec::util::*;
use pqx::ec::*;
use pqx::error::PqxResult;

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

    let arg = CmdArg::Ping { addr: "github.com" };
    let res = executor.exec(1, arg).await;

    println!("never reach!");
    assert!(res.is_ok());
    println!("{:?}", res.unwrap());
}

#[tokio::test]
async fn cmd_compose_and_exec_success2() {
    let py = cmd_which("python3").unwrap();
    let py = py.strip_suffix("\n").unwrap();

    println!("{:?}", py);
    let CmdChild { child_stdout, .. } = gen_bash_cmd(&format!(
        "cd {:?}/../scripts && {py} -u print_csv_in_line.py",
        std::env::current_dir().unwrap()
    ))
    .unwrap();

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

    let cmd = format!(
        "cd {} && {py} -u print_csv_in_line.py",
        dir.to_str().unwrap()
    );

    let arg = CmdArg::Bash { cmd: &cmd };
    let res = executor.exec(1, arg).await;

    assert!(res.is_ok());
    println!("{:?}", res.unwrap());
}

#[tokio::test]
async fn cmd_compose_and_exec_success3() {
    let py = cmd_which("python3").unwrap();
    let py = py.strip_suffix("\n").unwrap();
    let dir = join_dir(parent_dir(current_dir().unwrap()).unwrap(), "scripts").unwrap();

    let CmdChild { child_stdout, .. } = gen_bash_cmd(&format!(
        "cd {}/../scripts && {py} -u print_csv_in_line.py",
        dir.to_str().unwrap()
    ))
    .unwrap();

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

    let cmd = format!(
        "cd {:?}/../scripts && {py} -u print_csv_in_line.py",
        dir.to_str().unwrap()
    );

    let arg = CmdArg::Bash { cmd: &cmd };
    let res = executor.exec(1, arg).await;

    assert!(res.is_ok());
    println!("{:?}", res.unwrap());
}

#[tokio::test]
async fn cmd_compose_and_exec_success4() {
    let conda_env = "py310";
    let dir = join_dir(parent_dir(current_dir().unwrap()).unwrap(), "scripts").unwrap();
    let script = "print_csv_in_line.py";

    let CmdChild { child_stdout, .. } =
        gen_conda_python_cmd(conda_env, dir.to_str().unwrap(), script).unwrap();

    let (so_tx, so_rx) = gen_async_execution(1, child_stdout.into(), Arc::new(a_print_stdout));

    let task = tokio::try_join!(so_tx, so_rx);

    assert!(task.is_ok());
}

#[tokio::test]
async fn cmd_executor_success4() {
    let mut executor = CmdAsyncExecutor::new();
    executor.register_stdout_fn(Arc::new(a_print_stdout));
    executor.register_stderr_fn(Arc::new(a_print_stderr));

    let conda_env = "py310";
    let dir = join_dir(parent_dir(current_dir().unwrap()).unwrap(), "scripts").unwrap();
    let script = "print_csv_in_line.py";

    let arg = CmdArg::CondaPython {
        env: conda_env,
        dir: dir.to_str().unwrap(),
        script,
    };
    let res = executor.exec(1, arg).await;

    assert!(res.is_ok());
    println!("{:?}", res.unwrap());
}
