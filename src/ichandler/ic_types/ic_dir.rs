use diesel::MysqlConnection;
use crate::ichandler::ic_types::ic_packet::ic_packet;
use crate::ichandler::ic_types::ic_execute::ic_execute;
use crate::delete_dir;
use crate::show_dirs;
use crate::create_dir;
use crate::update_dir;

pub struct ic_dir { cmd: Vec<String>, }
impl ic_dir {
	pub fn new(args: Vec<String>) -> ic_dir {
		ic_dir { cmd: args }
	}
}
impl ic_execute for ic_dir {
	type Connection = MysqlConnection;
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> ic_packet {
		let mut create = false;
		let mut set = false;
		let mut delete = false;
		let mut show = false;
		let mut retstr: String = "OK.\n".to_string();
		match self.cmd[0].as_str() {
		"DELETE" => delete = true,
		"SHOW" => show = true,
		"CREATE" => create = true,
		"SET" => set = true,
		_ => eprintln!("{} is not a valid subcommand of DIR",self.cmd[0]),
		}

		
		if create {
			//CREATE ((NAME))
			if self.cmd.len() == 2 {
				create_dir(con.as_ref().unwrap(),&self.cmd[1],None);
			} else if ( self.cmd.len() == 4 ) {
				//CREATE ((NAME)) UNDER <DIR ID>
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
			//SET <ID> <DIR ID> <NEW NAME>
			if self.cmd.len() == 3 {
				println!("TARGET REACHED");
				update_dir(con.as_ref().unwrap(),self.cmd[1].parse::<i32>().unwrap(),self.cmd[2].parse::<i32>().unwrap(),None);
			} //ELSE update name as well 
		}
		ic_packet::new(Some("OK!".to_string()),Some(retstr.as_bytes().to_vec()))
	}
}
