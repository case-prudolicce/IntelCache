use diesel::MysqlConnection;
use intel_cache_lib::ic_types::ic_execute_mod::IcExecute;
use intel_cache_lib::ic_types::IcConnection;
use intel_cache_lib::ic_types::IcPacket;
use intel_cache_lib::ic_types::ic_connection_mod::IcLoginDetails;
use sha2::{Sha256, Digest};
use std::time::{SystemTime,UNIX_EPOCH};
use futures::executor::block_on;
use intel_cache_lib::lib_backend::establish_connection;
use intel_cache_lib::lib_backend::register;

pub struct CoreRegister {}
impl CoreRegister {
	#[no_mangle]
	pub fn cr_new() -> CoreRegister {
		CoreRegister { }
	}
	
	#[no_mangle]
	pub fn cr_to_exe() -> Box<dyn IcExecute<Connection = IcConnection>> {
		Box::new(CoreRegister::cr_new())
	}
}
impl IcExecute for CoreRegister {
	type Connection = IcConnection;
	
	fn exec(&mut self,con: &mut Self::Connection,cmd: Option<Vec<String>>,_data: Option<Vec<u8>>) -> IcPacket {
		match &cmd {
			Some(cmd) => {
				let username = &cmd[1];
				let pass = &cmd[2];
				let start = SystemTime::now();
				let since_the_epoch = start
					.duration_since(UNIX_EPOCH)
					.expect("Time went backwards")
					.as_secs().to_string();
				let mut hasher = Sha256::new();
				let pip: String;
				if let Some(ip) = block_on(public_ip::addr()) {
					pip = format!("{:?}", ip);
				} else {
					panic!("couldn't get an IP address");
				}
				let gid = username.to_owned()+pass+&since_the_epoch+&pip;
				hasher.update(&gid);
				let globalid = format!("{:x}",hasher.finalize());
				println!("{}->{}", gid,globalid);
				if pass.len() == 128 {
					register(&con.backend_con,&mut con.login,username.to_string(),pass.to_string(),globalid);
				}else {
					return IcPacket::new_empty()
				}
				return IcPacket::new_empty()
			}
			None => return IcPacket::new_empty(),
		}
	}
	
	fn login_required(&mut self) -> bool {
		false
	}
}
