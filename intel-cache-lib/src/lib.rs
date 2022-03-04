//! # IntelCache Library
//! 
//! The IntelCache Library is meant primarily for rust made clients of IntelCache and the server.
//! It functions primarily by sending and recieving [`self::ic_types::IcPacket`]s  to and from [`IcServer`] with [`IcClient`].
//!
//! To view available commands, check the apropriate storage module.
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

extern crate dotenv;
extern crate ipfs_api_backend_hyper;

pub mod ic_types;
pub mod lib_backend;
mod ic_client;
mod ic_server;
mod ic_module;

pub use self::ic_server::IcServer as IcServer;
pub use self::ic_client::IcClient as IcClient;
pub use self::ic_module::IcModule as IcModule;
