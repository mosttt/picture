#![feature(async_closure)]
#![feature(async_fn_in_trait)]
use crate::error::PError;

pub mod error;
pub mod handle;
pub mod picture;
pub mod utils;

pub type Result<T> = anyhow::Result<T, PError>;
