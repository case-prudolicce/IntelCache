//! # IntelCache Library
//! 
//! The IntelCache Library is meant primarily for rust made clients of IntelCache. 
//! It functions primarily by sending and recieving [`self::ic_types::IcCommand`]s and [`self::ic_types::IcPacket`]s  to and from [`IcServer`] with [`IcClient`].
//! # IntelCache Commands
//!
//! Here is the list of valid IntelCache commands:
//! - **`ENTRY {CREATE <NEW ENTRY NAME> [UNDER <DIR ID>]|SHOW [<DIR ID>]|DELETE <ENTRY ID>|SET <ENTRY ID> <DIR ID>|GET <ENTRY ID>}`**
//!     - **`ENTRY CREATE <NEW ENTRY NAME> [UNDER <DIR ID>]`**
//!
//!         This is for creating new entries with name `<NEW ENTRY NAME>`. The Command body will be the data to use for the new
//!         enty.
//!
//!         `UNDER <DIR ID>` Will create the entry with loc `<DIR ID>`. If it is missing, will default to 1.
//!     - **`ENTRY SHOW [<DIR ID>]`**
//!         
//!         This is to return entry summaries in IntelCache. If `<DIR ID>` is specified, only return summaries with loc `<DIR ID>`
//!         
//!     - **`ENTRY DELETE <ENTRY ID>`**
//!         
//!         This command deletes entry with id `<ENTRY ID>`.
//!         
//!     - **`ENTRY SET <ENTRY ID> <DIR ID>`**
//!         
//!         This command will change the loc of an entry with id `<ENTRY ID>` to loc `<DIR ID>`
//!
//!         It will also change the data of the entry if data is in the body.
//!         
//!     - **`ENTRY GET <ENTRY ID>`**
//!         
//!         This command will return an entry with id `<ENTRY ID>` with body containing data.
//! - **`DIR {CREATE <NEW DIR NAME> [UNDER <DIR ID>]|SHOW [<DIR ID>]|DELETE <DIR ID>|SET <DIR ID> <NEW DIR LOC ID>|VALIDATE <DIR ID>}`**
//!     - **`DIR CREATE <NEW DIR NAME> [UNDER <DIR ID>]`**
//!
//!         This is for creating new directories with name `<NEW DIR NAME>`. 
//!
//!         `UNDER <DIR ID>` Will create the entry with loc `<DIR ID>`. 
//!         If it is missing, the loc will be null (or commonly put, it will have no loc).
//!     - **`DIR SHOW [<DIR ID>]`**
//!
//!         This command will show all directories in the IntelCache if `<DIR ID>` is missing. If it isn't,
//!         it will show all directories in `<DIR ID>`
//!     - **`DIR DELETE <DIR ID>`**
//!
//!         This command will delete a directory with id `<DIR ID>`
//!     - **`DIR SET <DIR ID> <NEW DIR LOC ID>`**
//!
//!         This command will change a directory's loc (with id `<DIR ID>`) to a directory with id `<NEW DIR LOC ID>`
//!     - **`DIR VALIDATE <DIR ID>`**
//!
//!         This command will return `true` if `<DIR ID>` is a valid one (`false` if invalid), with it's name in the response's body.
//! - **`SHOW [<DIR ID>]`**
//!
//!     This Command will return all directories and entries in the IntelCache. If `<DIR ID>` is specified, it will return all on the specific directory id.
//! - **`TAG {DIR <DIR ID> <TAG ID>|UNDIR <DIR ID> <TAG ID>|ENTRY <ENTRY ID> <TAG ID>|UNENTRY <ENTRY ID> <TAG ID>|CREATE <NEW TAG NAME>|DELETE <TAG ID>|SHOW}`**
//!     - **`TAG DIR <DIR ID> <TAG ID>`**
//!
//!         This command will add a tag to a directory with id `<DIR ID>` with a tag with id `<TAG ID>`
//!
//!     - **`TAG UNDIR <DIR ID> <TAG ID>`**
//!
//!         This command will remove a tag with id `<TAG ID>` from a directory with id `<DIR ID>`
//!
//!     - **`TAG ENTRY <ENTRY ID> <TAG ID>`**
//!
//!         This command will add a tag to an entry with id `<ENTRY ID>` with a tag with id `<TAG ID>`
//!
//!     - **`TAG UNENTRY <ENTRY ID> <TAG ID>`**
//!
//!         This command will remove a tag with id `<TAG ID>` from an entry with id `<ENTRY ID>`
//!
//!     - **`TAG CREATE <NEW TAG NAME>`**
//!
//!         This command will create a tag with name `<NEW TAG NAME>`
//!
//!     - **`TAG DELETE <TAG ID>`**
//!
//!         This command will delete a tag with id `<TAG ID>`
//!
//!     - **`TAG SHOW`**
//!
//!         This command will return all available tags in the response body.
//!
//! - **`EXIT`**
//!
//!     This Command will disconnect the client from the IntelCache node
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

pub use self::ic_server::IcServer as IcServer;
pub use self::ic_client::IcClient as IcClient;
