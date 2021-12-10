use diesel::MysqlConnection;
use crate::untag_entry;
use crate::tag_entry;
use crate::untag_dir;
use crate::tag_dir;
use crate::create_tag;
use crate::ichandler::ic_types::ic_response::ic_response;
use crate::show_tags;
use crate::delete_tag;
use crate::ichandler::ic_types::ic_execute::ic_execute;

pub struct ic_tag {cmd: Vec<String>,}
impl ic_tag {
	pub fn new(args: Vec<String>) -> ic_tag {
		ic_tag { cmd: args }
	}
}
impl ic_execute for ic_tag {
	type Connection = MysqlConnection;
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> ic_response {
		let mut delete = false;
		let mut show = false;
		let mut create = false;
		let mut tagdir = 0;
		let mut tagentry = 0;
		let mut rstr = "OK.\n".to_string();

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
				delete_tag(&con.as_ref().unwrap(), (&self.cmd[1]).parse::<i32>().unwrap());
			}
		}

		if show {
			rstr = show_tags(&con.as_ref().unwrap(),Some(true));
			//return (if rstr.len() != 0 {Some(rstr.len() as i32)} else {None},if rstr.len() != 0 {Some(rstr)} else {None});
			return ic_response::from_str(rstr);
		}

		if create {
			//CREATE <TAG>
			if self.cmd.len() == 2 {
				create_tag(&con.as_ref().unwrap(), &self.cmd[1]);
			}
		}

		if tagdir == 1{
			//DIR <DIRID> <TAGID>
			if self.cmd.len() == 3 {
				tag_dir(&con.as_ref().unwrap(), (&self.cmd[1]).parse::<i32>().unwrap(),(&self.cmd[2]).parse::<i32>().unwrap());
			}
		} else if tagdir == -1 {
			//UNDIR <DIRID> <TAGID>
			if self.cmd.len() == 3 {
				untag_dir(&con.as_ref().unwrap(), (&self.cmd[1]).parse::<i32>().unwrap(),(&self.cmd[2]).parse::<i32>().unwrap());
			}
		}

		if tagentry == 1{
			if self.cmd.len() == 3 {
				tag_entry(&con.as_ref().unwrap(), (&self.cmd[1]).parse::<i32>().unwrap(),(&self.cmd[2]).parse::<i32>().unwrap());
			}
		} else if tagentry == -1 {
			if self.cmd.len() == 3 {
				untag_entry(&con.as_ref().unwrap(), (&self.cmd[1]).parse::<i32>().unwrap(),(&self.cmd[2]).parse::<i32>().unwrap());
			}
		}
		//(Some(4),Some("OK.\n".to_string()))
		ic_response::from_str(rstr)
	}
}
