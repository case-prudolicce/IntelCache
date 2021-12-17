use diesel::MysqlConnection;
use crate::ichandler::ic_types::IcPacket;
use crate::ichandler::ic_types::IcExecute;
use crate::ichandler::lib_backend::delete_dir;
use crate::ichandler::lib_backend::show_dirs;
use crate::ichandler::lib_backend::create_dir;
use crate::ichandler::lib_backend::update_dir;
use crate::ichandler::lib_backend::validate_dir;

pub struct IcDir { cmd: Vec<String>, }
impl IcDir {
	pub fn new(args: Vec<String>) -> IcDir {
		IcDir { cmd: args }
	}
}
impl IcExecute for IcDir {
	type Connection = MysqlConnection;
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> IcPacket {
		let mut create = false;
		let mut set = false;
		let mut delete = false;
		let mut show = false;
		let mut validate = false;
		let mut retstr: String = "".to_string();
		match self.cmd[0].as_str() {
		"DELETE" => delete = true,
		"SHOW" => show = true,
		"CREATE" => create = true,
		"SET" => set = true,
		"VALIDATE" => validate = true,
		_ => eprintln!("{} is not a valid subcommand of DIR",self.cmd[0]),
		}

		
		if create {
			if self.cmd.len() == 2 {
				create_dir(con.as_ref().unwrap(),&self.cmd[1],None);
			} else if self.cmd.len() == 4 {
				if self.cmd[2] == "UNDER" {
					create_dir(con.as_ref().unwrap(),&self.cmd[1],Some(self.cmd[3].parse::<i32>().unwrap()));
				} 
			}
		}
		if show {
			if self.cmd.len() == 2 {
				retstr = show_dirs(con.as_ref().unwrap(),Some(self.cmd[1].parse::<i32>().unwrap()))
			} else {
				retstr = show_dirs(con.as_ref().unwrap(),None)
			}
		}
		if delete {
			if self.cmd.len() == 2 {
				delete_dir(con.as_ref().unwrap(),self.cmd[1].parse::<i32>().unwrap());
			}
		}
		if set {
			if self.cmd.len() == 3 {
				update_dir(con.as_ref().unwrap(),self.cmd[1].parse::<i32>().unwrap(),self.cmd[2].parse::<i32>().unwrap(),None);
			}
		}
		if validate {
			let n = validate_dir(con.as_ref().unwrap(),self.cmd[1].parse::<i32>().unwrap());
			if n != None {
				return IcPacket::new(Some("true".to_string()),Some(n.unwrap().as_bytes().to_vec()));
			} else {
				return IcPacket::new(Some("false".to_string()),None);
			}
			
		}
		IcPacket::new(Some("OK!".to_string()),Some(retstr.as_bytes().to_vec()))
	}
}
