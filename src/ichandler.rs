pub mod ic_client;
pub mod ic_server;

use ic_server::ic_response;
pub trait ic_execute {
	type Connection;
	
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> ic_response;
}
