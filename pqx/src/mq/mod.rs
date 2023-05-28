//! file: mod.rs
//! author: Jacob Xie
//! date: 2023/05/26 23:52:32 Friday
//! brief:

pub mod client;
pub mod consumer;
pub mod publish;
pub mod subscribe;

macro_rules! get_connection {
    ($s:ident) => {
        $s.connection.as_ref().ok_or("connection is empty")
    };
}

macro_rules! get_channel {
    ($s:ident) => {
        $s.channel.as_ref().ok_or("channel is empty")
    };
}

pub(crate) use get_channel;
pub(crate) use get_connection;
