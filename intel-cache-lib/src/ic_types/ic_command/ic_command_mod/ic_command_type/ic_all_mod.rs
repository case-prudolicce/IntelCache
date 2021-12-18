use diesel::MysqlConnection;
use crate::ic_types::IcPacket;
use crate::lib_backend::show_entries;
use crate::lib_backend::show_dirs;
use crate::ic_types::IcExecute;

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
		if self.cmd.len() == 1 {
			retstr = show_dirs(con.as_ref().unwrap(),Some(self.cmd[0].parse::<i32>().unwrap()));
			retstr += &show_entries(con.as_ref().unwrap(),Some(false),Some(true),Some(self.cmd[0].parse::<i32>().unwrap()));
		} else {
			retstr = show_dirs(con.as_ref().unwrap(),None);
			retstr += &show_entries(con.as_ref().unwrap(),Some(false),Some(true),None);
		}
		IcPacket::new(Some("OK!".to_string()),Some(retstr.as_bytes().to_vec()))
	}
}
