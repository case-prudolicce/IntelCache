use intel_cache_lib::ic_types::ic_execute_mod::IcExecute;
use intel_cache_lib::ic_types::IcConnection;
use intel_cache_lib::ic_types::IcPacket;
use intel_cache_lib::lib_backend::rename_account;
use intel_cache_lib::lib_backend::change_password;
use intel_cache_lib::lib_backend::logout;
use intel_cache_lib::lib_backend::validate_user;
use intel_cache_lib::ic_types::IcLoginDetails;

pub struct CoreAccount {}
impl CoreAccount {
	#[no_mangle]
	pub fn ca_new() -> CoreAccount {
		CoreAccount { }
	}
	
	#[no_mangle]
	pub fn ca_to_exe() -> Box<dyn IcExecute<Connection = IcConnection>> {
		Box::new(CoreAccount::ca_new())
	}
}

impl IcExecute for CoreAccount {
	type Connection = IcConnection;
	
	fn exec(&mut self,con: &mut Self::Connection,cmd: Option<Vec<String>>,_data: Option<Vec<u8>>,_cached: bool) -> IcPacket {
		match &cmd {
			Some(cmd) => {
				if cmd[cmd.len() - 1..][0] == con.login.as_ref().unwrap_or(&IcLoginDetails {username: "ANONYMOUS".to_string(),id: "NONE".to_string(),cookie: "NONE".to_string()}).cookie {
					let mut rename = false;
					let mut chpwd = false;
					let mut lo = false;
					let mut validate = false;
					match cmd[1].as_ref() {
						"RENAME" => rename = true,
						"CHPASSWD" => chpwd = true,
						"LOGOUT" => lo = true,
						"VALIDATE" => validate = true,
						_=> return IcPacket::new(Some(cmd[1].clone()),None),
					};
					if rename {
						match rename_account(con,&cmd[2]) { 
							Ok(v) => return IcPacket::new(Some(v),None), 
							Err(_e) => return IcPacket::new(Some("Err.".to_string()),None), 
						};
					} else if chpwd {
						match change_password(con,&cmd[2]) {
							Ok(v) => return IcPacket::new(Some(v),None), 
							Err(_e) => return IcPacket::new(Some("Err.".to_string()),None), 
						};
					} else if lo {
						match logout(con) {
							Ok(v) => return IcPacket::new(Some(v),None), 
							Err(_e) => return IcPacket::new(Some("Err.".to_string()),None), 
						};
					} else if validate {
						match validate_user(con,&cmd[cmd.len() - 1..][0]) {
							Ok(v) => return IcPacket::new(Some(v),None), 
							Err(_e) => return IcPacket::new(Some("Err.".to_string()),None), 
						}
					} else { return IcPacket::new_empty(); }
				} else { return IcPacket::new_empty(); }
			}
			None => return IcPacket::new_empty(),
		}
	}
	
	fn login_required(&mut self) -> bool {
		false
	}
}
