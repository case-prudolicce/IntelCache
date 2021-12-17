use diesel::MysqlConnection;
use crate::ichandler::lib_backend::establish_connection;
use std::fmt::Display;
use std::fmt;
use crate::ichandler::ic_types::ic_execute;
use crate::ichandler::ic_types::ic_null;
use crate::ichandler::ic_types::ic_all;
use crate::ichandler::ic_types::ic_dir;
use crate::ichandler::ic_types::ic_tag;
use crate::ichandler::ic_types::ic_entry;
use crate::ichandler::ic_types::ic_packet;
use std::str;

#[derive(Clone)]
pub struct ic_command { pub cmd: Vec<String>,pub data: Vec<u8> }
impl ic_command {
	pub fn from_packet(p: ic_packet) -> ic_command {
		let mut proto_iccmd = ic_command { cmd:Vec::new(),data: p.body.unwrap_or(Vec::new()) };
		proto_iccmd.from_string(p.header.unwrap_or("".to_string()));
		proto_iccmd
	}
	pub fn new(raw_cmd: String) -> ic_command {
		let mut proto_iccmd = ic_command { cmd: Vec::new(), data: Vec::new() };
		proto_iccmd.from_string(raw_cmd);
		proto_iccmd
	}

	pub fn from_string(&mut self, s: String) {
		self.cmd = self.finalize_command(s.split_whitespace().collect::<Vec<&str>>());
	}

	fn finalize_command(&self,cmd: Vec<&str>) -> Vec<String> {
		//check for ((tokens That are included between these))
		//If found, concat to one str
		let mut con = false;
		let mut finalizedstr = String::new();
		let mut retve: Vec<String> = Vec::new();
		for c in cmd {
			if ! con { 
				if c.len() > 1 {
					if &c[..2] == "((" && ! (&c[c.len() - 2..] == "))"){ 
						con = true; 
						finalizedstr.push_str(&c[2..].to_string());
					} else {
						retve.push(c.to_string()); 
					}
				} else { retve.push(c.to_string()) }
			} else { 
				if c.len() > 1 {
					if &c[c.len() - 2..] == "))" {
						finalizedstr.push(' ');
						finalizedstr.push_str(&c[..c.len() - 2]);
						retve.push(finalizedstr);
						finalizedstr = String::new();
						con = false 
					}else { 
						finalizedstr.push(' ');
						finalizedstr.push_str(c);
					} 
				} else { finalizedstr.push(' '); finalizedstr.push_str(c) }
			}
		}
		retve
	}
	
	pub fn from_formated_vec(input: Vec<String>,d: Option<Vec<u8>>) -> ic_command {
		ic_command { cmd:input,data: d.unwrap_or(Vec::new()) }
	}

	pub fn parse(&self) -> Box<dyn ic_execute<Connection = MysqlConnection>> {
		//Returns an ic_execute by parsing cmd
		//0: null
		//1: dir
		//2: unbaked_entry
		//3: tag
		//4: show
		//-1: exit
		if self.cmd.len() <= 0 {return Box::new(ic_null::new())}
		let mut return_type = 0;
		match self.cmd[0].as_str() {
		"DIR" => return_type = 1,
		"ENTRY" => return_type = 2,
		"TAG" => return_type = 3,
		"SHOW" => return_type = 4,
		"EXIT" => return_type = -1,
		_ => (),
		}

		if return_type == 0 {
			return Box::new(ic_null::new());
		} else if return_type == -1 {
			return Box::new(ic_null::new());
		} else if return_type == 1 {
			return Box::new(ic_dir::new(self.cmd[1..].to_vec()));
		} else if return_type == 2 {
			return Box::new(ic_entry::from_ic_command(self.clone()));
		} else if return_type == 3 {
			return Box::new(ic_tag::new(self.cmd[1..].to_vec()));
		} else if return_type == 4 {
			return Box::new(ic_all::new(self.cmd[1..].to_vec()));
		} else {
			return Box::new(ic_null::new());
		}
	}
	
	pub fn to_ic_packet(&self) -> ic_packet {
		let mut s = String::new();
		for t in &self.cmd {
			if t.contains(char::is_whitespace) {
				s.push_str(&("((".to_owned()+&t+")) "));
			}else {s.push_str(&(t.to_owned()+" "))}
		}
		s = s.trim_end().to_string();
		ic_packet::new(Some(s),Some(self.clone().data))
	}
}
impl ic_execute for ic_command {
	type Connection = MysqlConnection;
	
	fn exec(&mut self,_con: Option<&mut Self::Connection>) -> ic_packet {
		handle(self.clone())
	}
}
impl Display for ic_command {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = String::new();
		//println!("ic_command#to_string: cmd is ({:?})",self.cmd);
		for c in &self.cmd {
			s.push_str(&c);
			s.push(' ');
		}
		write!(f,"{}", s)
	}
}

#[tokio::main]
pub async fn handle(cmd_opts: ic_command) -> ic_packet {
	let mut connection = establish_connection();
	let mut cmd_parsed = cmd_opts.parse();
	cmd_parsed.exec(Some(&mut connection))
}

