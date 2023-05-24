//! file: test_cmd.rs
//! author: Jacob Xie
//! date: 2023/05/22 23:15:04 Monday
//! brief:

use pqx::cmd::{gen_async_execution, gen_ping_cmd, CmdArg, CmdExecutor};

fn print_stdout(s: String) {
    println!("print_stdout: {:?}", s);
}

fn print_stderr(s: String) {
    println!("print_stderr: {:?}", s);
}

#[tokio::test]
async fn cmd_compose_and_exec_success() {
    let (_, co, _) = gen_ping_cmd("github.com").unwrap();

    let (so_tx, so_rx) = gen_async_execution(1, co.into(), print_stdout);

    let task = tokio::try_join!(so_tx, so_rx);

    println!("never reach!");
    assert!(task.is_ok());
}

#[tokio::test]
async fn cmd_executor_success() {
    let mut executor = CmdExecutor::new();
    executor.register_stdout_fn(print_stdout);
    executor.register_stderr_fn(print_stderr);

    let arg = CmdArg::Ping { addr: "github.com" };
    let res = executor.exec(1, arg).await;

    println!("never reach!");
    assert!(res.is_ok());
    println!("{:?}", res.unwrap());
}
