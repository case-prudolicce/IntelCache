use crate::ichandler::ic_types::*;
use diesel::MysqlConnection;

use crate::ichandler::ic_types::ic_response::ic_response;
use crate::ichandler::ic_types::ic_execute::ic_execute;

pub struct ic_null {}
impl ic_null {
	pub fn new() -> ic_null {
		ic_null { }
	}
}
impl ic_execute for ic_null {
	type Connection = MysqlConnection;
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> ic_response {
		ic_response::null_response()
	}
}
