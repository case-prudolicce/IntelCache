use diesel::MysqlConnection;
use crate::ic_types::ic_execute_mod::IcExecute;
use crate::ic_types::IcConnection;
use crate::ic_types::IcPacket;
use crate::ic_types::ic_connection_mod::IcLoginDetails;
use sha2::{Sha256, Digest};
use std::time::{SystemTime,UNIX_EPOCH};
use futures::executor::block_on;
use crate::lib_backend::establish_connection;
use crate::lib_backend::register;

pub struct IcRegister {cmd: Vec<String>,}
impl IcRegister {
	pub fn new(args: Vec<String>) -> IcRegister {
		IcRegister { cmd: args }
	}
}
impl IcExecute for IcRegister {
	type Connection = MysqlConnection;
	type LoginDetails = Option<IcLoginDetails>;
	
	fn exec(&mut self,con: Option<&mut Self::Connection>,login: &mut Self::LoginDetails) -> IcPacket {
		let username = &self.cmd[1];
		let pass = &self.cmd[2];
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
			let con: MysqlConnection;
			match establish_connection() {
			Ok(v) => con = v,
			Err(e) => panic!("{:?}",e),
			}
			register(&con,login,username.to_string(),pass.to_string(),globalid);
		}else {
			return IcPacket::new_empty()
		}
		return IcPacket::new_empty()
	}
	
	fn login_required(&mut self) -> bool {
		false
	}
}
