use diesel::MysqlConnection;
use crate::ichandler::ic_types::ic_packet::ic_packet;
use crate::show_entries;
use crate::show_dirs;
use crate::ichandler::ic_types::ic_execute::ic_execute;

pub struct ic_all { cmd: Vec<String>, }
impl ic_all {
	pub fn new(args: Vec<String>) -> ic_all {
		ic_all { cmd: args }
	}
}
impl ic_execute for ic_all {
	type Connection = MysqlConnection;
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> ic_packet {
		let mut retstr: String = "OK.\n".to_string();
		if self.cmd.len() == 1 {
			retstr = show_dirs(con.as_ref().unwrap(),Some(self.cmd[0].parse::<i32>().unwrap()));
			retstr += &show_entries(con.as_ref().unwrap(),Some(false),Some(true));
		} else {
			retstr = show_dirs(con.as_ref().unwrap(),None);
			retstr += &show_entries(con.as_ref().unwrap(),Some(false),Some(true));
		}
		ic_packet::new(Some("OK!".to_string()),Some(retstr.as_bytes().to_vec()))
	}
}
