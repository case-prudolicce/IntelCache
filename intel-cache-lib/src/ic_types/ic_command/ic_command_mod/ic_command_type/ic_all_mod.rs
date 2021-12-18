use diesel::MysqlConnection;
use crate::ic_types::IcPacket;
use crate::lib_backend::show_entries;
use crate::lib_backend::show_dirs;
use crate::lib_backend::validate_dir;
//use crate::ic_types::IcExecute;
use crate::ic_types::ic_execute_mod::IcExecute;

pub struct IcAll { cmd: Vec<String>, }
impl IcAll {
	pub fn new(args: Vec<String>) -> IcAll {
		IcAll { cmd: args }
	}
}
impl IcExecute for IcAll {
	type Connection = MysqlConnection;
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> IcPacket {
		let mut retstr: String;
		if self.cmd.len() > 1 {
			let si = match self.cmd[1].parse::<i32>() {
			Ok(v) => if v == 0 {None} else {
				match validate_dir(con.as_ref().unwrap(),v) {
				Some(_iv) => Some(v),
				None => return IcPacket::new(Some("Err.".to_string()),None),
				}
			},
			Err(_err) => return IcPacket::new(Some("Err.".to_string()),None)
			};

			if self.cmd.len() == 2 && si != None {
				retstr = show_dirs(con.as_ref().unwrap(),Some(si.unwrap()));
				retstr += &show_entries(con.as_ref().unwrap(),Some(false),Some(true),Some(si.unwrap()));
			} else if self.cmd.len() == 2 {
				retstr = show_dirs(con.as_ref().unwrap(),None);
				retstr += &show_entries(con.as_ref().unwrap(),Some(false),Some(true),None);
			} else { return IcPacket::new(Some("Err.".to_string()),None) }
			IcPacket::new(Some("OK!".to_string()),Some(retstr.as_bytes().to_vec()))
		} else {
			retstr = show_dirs(con.as_ref().unwrap(),None);
			retstr += &show_entries(con.as_ref().unwrap(),Some(false),Some(true),None);
			IcPacket::new(Some("OK!".to_string()),Some(retstr.as_bytes().to_vec()))
		}
	}
}
