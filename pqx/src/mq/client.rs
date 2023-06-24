//! file: client.rs
//! author: Jacob Xie
//! date: 2023/05/27 09:18:35 Saturday
//! brief:

use std::sync::Arc;

use amqprs::callbacks::{ChannelCallback, ConnectionCallback};
use amqprs::channel::*;
use amqprs::connection::{Connection, OpenConnectionArguments};
use amqprs::FieldTable;
use pqx_util::{read_json, read_yaml};
use serde::{Deserialize, Serialize};

use super::{get_channel, get_connection, FieldTableBuilder, X_DELAYED_MESSAGE};
use crate::error::PqxResult;

// ================================================================================================
// Conn
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqConn {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub vhost: Option<String>,
}

// ================================================================================================
// SimpleQueueInfo
// ================================================================================================

#[derive(Debug)]
pub struct SimpleQueueInfo {
    pub name: String,
    pub message_count: u32,
    pub consumer_count: u32,
}

impl SimpleQueueInfo {
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

#[derive(Default, Clone)]
pub struct MqClient {
    connection: Option<Connection>,
    conn_callback: Option<Arc<dyn ConnectionCallback>>,
    chan_callback: Option<Arc<dyn ChannelCallback>>,
    channel: Option<Channel>,
}

impl MqClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn connect(&mut self, conn_arg: MqConn) -> PqxResult<()> {
        let MqConn {
            host,
            port,
            user,
            pass,
            vhost,
        } = conn_arg;

        let mut arg = OpenConnectionArguments::new(&host, port, &user, &pass);
        if let Some(vh) = vhost {
            arg.virtual_host(vh.as_ref());
        }

        self.connection = Some(Connection::open(&arg).await?);

        Ok(())
    }

    pub async fn connect_by_yaml(&mut self, path: impl AsRef<str>) -> PqxResult<()> {
        let cfg: MqConn = read_yaml(path)?;

        self.connect(cfg).await
    }

    pub async fn connect_by_json(&mut self, path: impl AsRef<str>) -> PqxResult<()> {
        let cfg: MqConn = read_json(path)?;

        self.connect(cfg).await
    }

    pub async fn disconnect(&mut self) -> PqxResult<()> {
        // ignore error
        let _ = self.close_channel().await;

        if let Some(c) = self.connection.take() {
            c.close().await?
        };

        Ok(())
    }

    pub fn connection(&self) -> PqxResult<&Connection> {
        let conn = get_connection!(self)?;

        Ok(conn)
    }

    pub fn channel(&self) -> PqxResult<&Channel> {
        let chan = get_channel!(self)?;

        Ok(chan)
    }

    pub fn register_conn_callback<C>(&mut self, callback: C)
    where
        C: ConnectionCallback + 'static,
    {
        self.conn_callback = Some(Arc::new(callback));
    }

    pub fn register_chan_callback<C>(&mut self, callback: C)
    where
        C: ChannelCallback + 'static,
    {
        self.chan_callback = Some(Arc::new(callback));
    }

    pub async fn open_channel(&mut self, id: Option<u16>) -> PqxResult<()> {
        let chan = if let Some(c) = &self.connection {
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

    pub async fn declare_exchange(
        &self,
        name: &str,
        exchange_type: &ExchangeType,
    ) -> PqxResult<()> {
        let chan = get_channel!(self)?;

        let mut args = ExchangeDeclareArguments::new(name, &exchange_type.to_string());
        args.durable(true);
        chan.exchange_declare(args.finish()).await?;

        Ok(())
    }

    pub async fn declare_delayed_exchange(
        &self,
        name: &str,
        exchange_type: &ExchangeType,
    ) -> PqxResult<()> {
        let mut args = FieldTableBuilder::new();
        args.x_delayed_type(exchange_type);
        self.declare_exchange_with_args(name, &X_DELAYED_MESSAGE, args.finish())
            .await?;

        Ok(())
    }

    pub async fn declare_exchange_with_args(
        &self,
        name: &str,
        exchange_type: &ExchangeType,
        args: FieldTable,
    ) -> PqxResult<()> {
        let chan = get_channel!(self)?;

        let exchange_args = ExchangeDeclareArguments::new(name, &exchange_type.to_string())
            .durable(true)
            .arguments(args)
            .finish();

        chan.exchange_declare(exchange_args).await?;

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

    pub async fn declare_queue(&self, que: &str) -> PqxResult<()> {
        let chan = get_channel!(self)?;

        chan.queue_declare(QueueDeclareArguments::durable_client_named(que))
            .await?;

        Ok(())
    }

    pub async fn declare_queue_by_args(&self, args: QueueDeclareArguments) -> PqxResult<()> {
        let chan = get_channel!(self)?;

        chan.queue_declare(args).await?;

        Ok(())
    }

    pub async fn declare_and_bind_queue(
        &self,
        exchange: &str,
        rout: &str,
        que: &str,
    ) -> PqxResult<SimpleQueueInfo> {
        let chan = get_channel!(self)?;

        let (name, message_count, consumer_count) = chan
            .queue_declare(QueueDeclareArguments::durable_client_named(que))
            .await?
            .unwrap();

        chan.queue_bind(QueueBindArguments::new(que, exchange, rout))
            .await?;

        Ok(SimpleQueueInfo::new(name, message_count, consumer_count))
    }

    pub async fn declare_queue_with_dlx(
        &self,
        que: &str,
        dlx: &str,
        dlx_rout: &str,
        ttl: Option<i64>,
    ) -> PqxResult<()> {
        let mut args = QueueDeclareArguments::durable_client_named(que);
        let mut ft = FieldTableBuilder::new();
        ft.x_dead_letter_exchange(dlx, dlx_rout);
        if let Some(t) = ttl {
            ft.x_message_ttl(t);
        }
        args.arguments(ft.finish());

        self.declare_queue_by_args(args).await?;

        Ok(())
    }

    pub async fn bind_queue_by_args(&self, args: QueueBindArguments) -> PqxResult<()> {
        let chan = get_channel!(self)?;

        chan.queue_bind(args).await?;

        Ok(())
    }

    pub async fn bind_queue(&self, exchange: &str, rout: &str, que: &str) -> PqxResult<()> {
        let chan = get_channel!(self)?;

        chan.queue_bind(QueueBindArguments::new(que, exchange, rout))
            .await?;

        Ok(())
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
