use crate::ichandler::ic_types::*;
use diesel::MysqlConnection;

use crate::ichandler::ic_types::ic_packet;
use crate::ichandler::ic_types::ic_execute;

pub struct ic_null {}
impl ic_null {
	pub fn new() -> ic_null {
		ic_null { }
	}
}
impl ic_execute for ic_null {
	type Connection = MysqlConnection;
	fn exec(&mut self,_con: Option<&mut Self::Connection>) -> ic_packet {
		ic_packet::new_empty()
	}
}
