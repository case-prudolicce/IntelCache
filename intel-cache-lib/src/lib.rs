//! # IntelCache Library
//! 
//! The IntelCache Library is meant primarily for rust made clients of IntelCache. 
//! It functions primarily by sending and recieving [`self::ic_types::IcCommand`]s and [`self::ic_types::IcPacket`]s  to and from [`IcServer`] with [`IcClient`].
//! # IntelCache Commands
//!
//! Here is the list of valid IntelCache commands:
//! - ENTRY {CREATE \<NEW ENTRY NAME\> [UNDER \<DIR ID\>]|SHOW [\<DIR ID\>]|DELETE \<ENTRY ID\>|SET \<ENTRY ID\> \<DIR ID\>}
//! - DIR {CREATE \<NEW DIR NAME\> [UNDER \<DIR ID\>]|SHOW [\<DIR ID\>]|DELETE \<DIR ID\>|SET \<DIR ID\> \<NEW DIR LOC ID\>|VALIDATE \<DIR ID\>}
//! - SHOW [\<DIR ID\>]
//! - TAG {DIR \<DIR ID\> \<TAG ID\>|UNDIR \<DIR ID\> \<TAG ID\>|ENTRY \<ENTRY ID\> \<TAG ID\>|UNENTRY \<ENTRY ID\> \<TAG ID\>|CREATE <NEW TAG NAME>|DELETE <TAG ID>|SHOW}
//! - EXIT
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate ipfs_api_backend_hyper;

pub mod ic_types;
mod lib_backend;
mod ic_client;
mod ic_server;

pub use self::ic_server::IcServer as IcServer;
pub use self::ic_client::IcClient as IcClient;
