use crate::ic_types::ic_command::ic_command_mod::IcExecute;
use diesel::MysqlConnection;
use crate::ic_types::IcLoginDetails;
use crate::ic_types::IcPacket;
use crate::lib_backend::login;
pub struct IcLogin {cmd: Vec<String>,}
impl IcLogin {
	pub fn new(args: Vec<String>) -> IcLogin {
		IcLogin { cmd: args }
	}
}
impl IcExecute for IcLogin {
	type Connection = MysqlConnection;
	type LoginDetails = Option<IcLoginDetails>;
	
	fn exec(&mut self,con: Option<&mut Self::Connection>,l: &mut Self::LoginDetails) -> IcPacket {
		let globalid = &self.cmd[1];
		let pass = &self.cmd[2];
		if pass.len() == 128 {
			let c: &mut MysqlConnection;
			match con {
			Some(connection) => c = connection,
			None => panic!("CONNECTION REQUIRED"),
			}
			match login(c,l,globalid.to_string(),pass.to_string()) {
				Ok(c) => return IcPacket::new(Some(c),None),
				Err(e) => return IcPacket::new_denied(),
			}
			
		}else {
			return IcPacket::new_denied();
		}
		return IcPacket::new_denied()
	}
	
	fn login_required(&mut self) -> bool {
		false
	}
}
