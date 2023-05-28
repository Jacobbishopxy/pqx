//! file: lib.rs
//! author: Jacob Xie
//! date: 2023/05/22 20:39:45 Monday
//! brief:

#![feature(impl_trait_in_assoc_type)]

pub mod ec;
pub mod error;
pub mod mq;

pub use amqprs::*;
