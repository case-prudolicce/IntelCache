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
