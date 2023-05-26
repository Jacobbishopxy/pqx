//! file: util.rs
//! author: Jacob Xie
//! date: 2023/05/26 23:52:53 Friday
//! brief:

use amqprs::callbacks::{DefaultChannelCallback, DefaultConnectionCallback};
use amqprs::channel::{Channel, QueueBindArguments, QueueDeclareArguments};
use amqprs::connection::{Connection, OpenConnectionArguments};

use crate::error::PqxResult;

/// Open a connection
pub async fn open_connection(
    host: &str,
    port: u16,
    user: &str,
    pass: &str,
    vhost: Option<&str>,
) -> PqxResult<Connection> {
    let mut arg = OpenConnectionArguments::new(host, port, user, pass);
    if let Some(vh) = vhost {
        arg.virtual_host(vh.as_ref());
    }

    let conn = Connection::open(&arg).await?;

    conn.register_callback(DefaultConnectionCallback).await?;

    Ok(conn)
}

/// Open a channel
pub async fn open_channel(conn: &Connection, channel_id: Option<u16>) -> PqxResult<Channel> {
    let channel = conn.open_channel(channel_id).await?;

    channel.register_callback(DefaultChannelCallback).await?;

    Ok(channel)
}

/// Declare a durable queue
/// Return: (queue_name, message_count, consumer_count)
pub async fn declare_queue(
    chan: &Channel,
    que: &str,
    rout: &str,
    exchg: &str,
) -> PqxResult<(String, u32, u32)> {
    let res = chan
        .queue_declare(QueueDeclareArguments::durable_client_named(que))
        .await?
        .unwrap();

    // bind que to an exchange
    chan.queue_bind(QueueBindArguments::new(que, exchg, rout))
        .await?;

    Ok(res)
}
