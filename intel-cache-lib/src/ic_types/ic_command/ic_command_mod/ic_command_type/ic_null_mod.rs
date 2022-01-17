use diesel::MysqlConnection;

use crate::ic_types::IcPacket;
//use crate::ic_types::IcExecute;
use crate::ic_types::ic_execute_mod::IcExecute;
use crate::ic_types::ic_connection_mod::IcLoginDetails;

pub struct IcNull {}
impl IcNull {
	pub fn new() -> IcNull {
		IcNull { }
	}
}
impl IcExecute for IcNull {
	type Connection = MysqlConnection;
	type LoginDetails = Option<IcLoginDetails>;
	fn exec(&mut self,_con: Option<&mut Self::Connection>,login: &mut Self::LoginDetails) -> IcPacket {
		IcPacket::new_empty()
	}
	
	fn login_required(&mut self) -> bool {
		false
	}
}
