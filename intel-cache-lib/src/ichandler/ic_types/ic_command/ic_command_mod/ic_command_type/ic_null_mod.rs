use diesel::MysqlConnection;

use crate::ichandler::ic_types::IcPacket;
use crate::ichandler::ic_types::IcExecute;

pub struct IcNull {}
impl IcNull {
	pub fn new() -> IcNull {
		IcNull { }
	}
}
impl IcExecute for IcNull {
	type Connection = MysqlConnection;
	fn exec(&mut self,_con: Option<&mut Self::Connection>) -> IcPacket {
		IcPacket::new_empty()
	}
}
