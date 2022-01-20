use intel_cache_lib::ic_types::ic_command::ic_command_mod::IcExecute;
use diesel::MysqlConnection;
use intel_cache_lib::ic_types::IcLoginDetails;
use intel_cache_lib::ic_types::IcPacket;
use intel_cache_lib::lib_backend::login;
pub struct CoreLogin {cmd: Option<Vec<String>>,}
impl CoreLogin {
	pub fn new() -> CoreLogin {
		CoreLogin { cmd: None }
	}
	
	pub fn load(&mut self,args: Vec<String>) {
		self.cmd = Some(args);
	}

	pub fn to_exe() -> Box<dyn IcExecute<Connection = MysqlConnection,LoginDetails = Option<IcLoginDetails>>> {
		Box::new(CoreLogin::new())
	}
}
impl IcExecute for CoreLogin {
	type Connection = MysqlConnection;
	type LoginDetails = Option<IcLoginDetails>;
	
	fn exec(&mut self,con: Option<&mut Self::Connection>,l: &mut Self::LoginDetails) -> IcPacket {
		match &self.cmd {
			Some(cmd) => {
				let globalid = &cmd[1];
				let pass = &cmd[2];
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
			},
			None => return IcPacket::new_denied(),
		}
	}
	
	fn login_required(&mut self) -> bool {
		false
	}
}
