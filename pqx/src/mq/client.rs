//! file: client.rs
//! author: Jacob Xie
//! date: 2023/05/27 09:18:35 Saturday
//! brief:

use amqprs::callbacks::{ChannelCallback, ConnectionCallback};
use amqprs::channel::*;
use amqprs::connection::{Connection, OpenConnectionArguments};

use super::get_channel;
use crate::error::PqxResult;

// ================================================================================================
// Conn
// ================================================================================================

pub struct ConnArg<'a> {
    pub host: &'a str,
    pub port: u16,
    pub user: &'a str,
    pub pass: &'a str,
    pub vhost: Option<&'a str>,
}

// ================================================================================================
// QueueInfo
// ================================================================================================

pub struct QueueInfo {
    pub name: String,
    pub message_count: u32,
    pub consumer_count: u32,
}

impl QueueInfo {
    pub fn new(name: String, message_count: u32, consumer_count: u32) -> Self {
        Self {
            name,
            message_count,
            consumer_count,
        }
    }
}

// ================================================================================================
// MqClient
// ================================================================================================

#[derive(Default)]
pub struct MqClient {
    conn: Option<Connection>,
    conn_callback: Option<Box<dyn ConnectionCallback>>,
    chan_callback: Option<Box<dyn ChannelCallback>>,
    channel: Option<Channel>,
}

impl MqClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn connect(&mut self, conn_arg: ConnArg<'_>) -> PqxResult<Self> {
        let ConnArg {
            host,
            port,
            user,
            pass,
            vhost,
        } = conn_arg;

        let mut arg = OpenConnectionArguments::new(host, port, user, pass);
        if let Some(vh) = vhost {
            arg.virtual_host(vh.as_ref());
        }

        let conn = Connection::open(&arg).await?;

        Ok(Self {
            conn: Some(conn),
            conn_callback: None,
            chan_callback: None,
            channel: None,
        })
    }

    pub async fn disconnect(&mut self) -> PqxResult<()> {
        // ignore error
        let _ = self.close_channel().await;

        if let Some(c) = self.conn.take() {
            c.close().await?
        };

        Ok(())
    }

    pub fn register_conn_callback<C>(&mut self, callback: C)
    where
        C: ConnectionCallback + 'static,
    {
        self.conn_callback = Some(Box::new(callback));
    }

    pub fn register_chan_callback<C>(&mut self, callback: C)
    where
        C: ChannelCallback + 'static,
    {
        self.chan_callback = Some(Box::new(callback));
    }

    pub async fn open_channel(&mut self, id: Option<u16>) -> PqxResult<()> {
        let chan = if let Some(c) = &self.conn {
            c.open_channel(id).await?
        } else {
            return Err("connection is empty".into());
        };

        self.channel = Some(chan);

        Ok(())
    }

    pub async fn close_channel(&mut self) -> PqxResult<()> {
        match self.channel.take() {
            Some(c) => {
                c.close().await?;
                self.channel = None;
                Ok(())
            }
            None => Err("channel is empty".into()),
        }
    }

    pub async fn declare_exchange(&self, name: &str, exchange_type: ExchangeType) -> PqxResult<()> {
        let chan = get_channel!(self)?;

        chan.exchange_declare(ExchangeDeclareArguments::new(
            name,
            &exchange_type.to_string(),
        ))
        .await?;

        Ok(())
    }

    pub async fn bind_exchange(
        &self,
        source_exchange: &str,
        target_exchange: &str,
        rout: &str,
    ) -> PqxResult<()> {
        let chan = get_channel!(self)?;

        chan.exchange_bind(ExchangeBindArguments::new(
            target_exchange,
            source_exchange,
            rout,
        ))
        .await?;

        Ok(())
    }

    pub async fn unbind_exchange(
        &self,
        source_exchange: &str,
        target_exchange: &str,
        rout: &str,
    ) -> PqxResult<()> {
        let chan = get_channel!(self)?;

        chan.exchange_unbind(ExchangeUnbindArguments::new(
            target_exchange,
            source_exchange,
            rout,
        ))
        .await?;

        Ok(())
    }

    pub async fn delete_exchange(&self, exchange: &str) -> PqxResult<()> {
        let chan = get_channel!(self)?;

        chan.exchange_delete(ExchangeDeleteArguments::new(exchange))
            .await?;

        Ok(())
    }

    pub async fn declare_queue(
        &self,
        exchange: &str,
        rout: &str,
        que: &str,
    ) -> PqxResult<QueueInfo> {
        let chan = get_channel!(self)?;

        let (name, message_count, consumer_count) = chan
            .queue_declare(QueueDeclareArguments::durable_client_named(que))
            .await?
            .unwrap();

        chan.queue_bind(QueueBindArguments::new(que, exchange, rout))
            .await?;

        Ok(QueueInfo::new(name, message_count, consumer_count))
    }

    pub async fn delete_queue(&self, que: &str) -> PqxResult<()> {
        let chan = get_channel!(self)?;

        chan.queue_delete(QueueDeleteArguments::new(que)).await?;

        Ok(())
    }

    pub async fn purge_queue(&self, que: &str) -> PqxResult<()> {
        let chan = get_channel!(self)?;

        chan.queue_purge(QueuePurgeArguments::new(que)).await?;

        Ok(())
    }

    pub async fn unbind_queue(&self, exchange: &str, rout: &str, que: &str) -> PqxResult<()> {
        let chan = get_channel!(self)?;

        chan.queue_unbind(QueueUnbindArguments::new(que, exchange, rout))
            .await?;

        Ok(())
    }
}
