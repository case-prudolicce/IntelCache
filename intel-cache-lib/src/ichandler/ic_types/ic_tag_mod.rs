use diesel::MysqlConnection;
use crate::ichandler::lib_backend::untag_entry;
use crate::ichandler::lib_backend::tag_entry;
use crate::ichandler::lib_backend::untag_dir;
use crate::ichandler::lib_backend::tag_dir;
use crate::ichandler::lib_backend::create_tag;
use crate::ichandler::ic_types::IcPacket;
use crate::ichandler::lib_backend::show_tags;
use crate::ichandler::lib_backend::delete_tag;
use crate::ichandler::ic_types::IcExecute;

pub struct IcTag {cmd: Vec<String>,}
impl IcTag {
	pub fn new(args: Vec<String>) -> IcTag {
		IcTag { cmd: args }
	}
}
impl IcExecute for IcTag {
	type Connection = MysqlConnection;
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> IcPacket {
		let mut delete = false;
		let mut show = false;
		let mut create = false;
		let mut tagdir = 0;
		let mut tagentry = 0;
		let mut rstr = "".to_string();

		match self.cmd[0].as_str() {
		"DELETE" => delete = true,
		"SHOW" => show = true,
		"CREATE" => create = true,
		"DIR" => tagdir = 1,
		"UNDIR" => tagdir = -1,
		"ENTRY" => tagentry = 1,
		"UNENTRY" => tagentry = -1,
		_ => panic!("{} is not a valid subcommand of TAG.",self.cmd[0]),
		}
		if delete {
			if self.cmd.len() == 2 {
				let ttd: i32;
				match (&self.cmd[1]).parse::<i32>() {
				Ok(e) => {ttd = e}
				Err(_err) => {return IcPacket::new(Some("Err.".to_string()),None)}
				}
				let r = delete_tag(&con.as_ref().unwrap(), ttd);
				match r {
				Ok(_v) => {return IcPacket::new(Some("OK!".to_string()),None)},
				Err(_e) => {return IcPacket::new(Some("Err.".to_string()),None)},
				}
			}
		}

		if show {
			rstr = show_tags(&con.as_ref().unwrap(),Some(true));
			return IcPacket::new(Some("OK!".to_string()),Some(rstr.as_bytes().to_vec()));
		}

		if create {
			if self.cmd.len() == 2 {
				create_tag(&con.as_ref().unwrap(), &self.cmd[1]);
			}
		}

		if tagdir == 1{
			if self.cmd.len() == 3 {
				let res = tag_dir(&con.as_ref().unwrap(), (&self.cmd[1]).parse::<i32>().unwrap(),(&self.cmd[2]).parse::<i32>().unwrap());
				match res {
				Ok(_e) => (),
				Err(_err) => {return IcPacket::new(Some("Err.".to_string()),None) }
				};
			}
		} else if tagdir == -1 {
			if self.cmd.len() == 3 {
				untag_dir(&con.as_ref().unwrap(), (&self.cmd[1]).parse::<i32>().unwrap(),(&self.cmd[2]).parse::<i32>().unwrap());
			}
		}

		if tagentry == 1{
			if self.cmd.len() == 3 {
				let res = tag_entry(&con.as_ref().unwrap(), (&self.cmd[1]).parse::<i32>().unwrap(),(&self.cmd[2]).parse::<i32>().unwrap());
				match res {
				Ok(_e) => (),
				Err(_err) => {return IcPacket::new(Some("Err.".to_string()),None) }
				};
			}
		} else if tagentry == -1 {
			if self.cmd.len() == 3 {
				untag_entry(&con.as_ref().unwrap(), (&self.cmd[1]).parse::<i32>().unwrap(),(&self.cmd[2]).parse::<i32>().unwrap());
			}
		}
		IcPacket::new(Some("Err.".to_string()),None)
	}
}
