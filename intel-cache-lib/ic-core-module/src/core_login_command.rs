use intel_cache_lib::ic_types::IcExecute;
use intel_cache_lib::ic_types::IcPacket;
use intel_cache_lib::lib_backend::login;
use intel_cache_lib::ic_types::IcConnection;

pub struct CoreLogin {}
impl CoreLogin {
	#[no_mangle]
	pub fn cl_new() -> CoreLogin {
		CoreLogin {}
	}
	
	#[no_mangle]
	pub fn cl_to_exe() -> Box<dyn IcExecute<Connection = IcConnection>> {
		Box::new(CoreLogin::cl_new())
	}
}
impl IcExecute for CoreLogin {
	type Connection = IcConnection;
	
	fn exec(&mut self,con: &mut Self::Connection,cmd: Option<Vec<String>>,_data: Option<Vec<u8>>) -> IcPacket {
		match cmd {
			Some(cmd) => {
				let globalid = &cmd[1];
				let pass = &cmd[2];
				if pass.len() == 128 {
					match login(&con.backend_con,&mut con.login,globalid.to_string(),pass.to_string()) {
						Ok(c) => return IcPacket::new(Some(c),None),
						Err(_e) => return IcPacket::new_denied(),
					}
					
				}else {
					return IcPacket::new_denied();
				}
			},
			None => return IcPacket::new_denied(),
		}
	}
	
	fn login_required(&mut self) -> bool {
		false
	}
}
