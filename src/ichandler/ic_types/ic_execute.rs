use diesel::MysqlConnection;
use crate::ichandler::ic_types::ic_response::ic_response;

pub trait ic_execute {
	type Connection;
	
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> ic_response;
}
