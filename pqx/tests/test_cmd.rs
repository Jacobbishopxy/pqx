//! file: test_cmd.rs
//! author: Jacob Xie
//! date: 2023/05/22 23:15:04 Monday
//! brief:

use pqx::cmd::{async_cmd, gen_ping_cmd};

fn print_stdout(s: String) {
    println!("print_stdout: {:?}", s);
}

#[tokio::test]
async fn test1() {
    let (mut c, co, _) = gen_ping_cmd("github.com").unwrap();

    async_cmd(1, Some((co, print_stdout)), None).await.unwrap();

    println!("never reach");

    c.wait().unwrap();
}
