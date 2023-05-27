//! file: mod.rs
//! author: Jacob Xie
//! date: 2023/05/26 23:52:32 Friday
//! brief:

pub mod client;
pub mod publish;
pub mod subscribe;

macro_rules! get_channel {
    ($s:ident) => {
        $s.channel.as_ref().ok_or("channel is empty")
    };
}

pub(crate) use get_channel;
