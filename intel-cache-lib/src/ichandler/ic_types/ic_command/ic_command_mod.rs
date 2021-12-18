use diesel::MysqlConnection;
use crate::ichandler::lib_backend::establish_connection;
use std::fmt::Display;
use std::fmt;
use crate::ichandler::ic_types::IcExecute;
use crate::ichandler::ic_types::IcPacket;
use std::str;

mod ic_command_type;
use self::ic_command_type::ic_null_mod::IcNull as IcNull;
use self::ic_command_type::ic_dir_mod::IcDir as IcDir;
use self::ic_command_type::ic_all_mod::IcAll as IcAll;
use self::ic_command_type::ic_entry_mod::IcEntry as IcEntry;
use self::ic_command_type::ic_tag_mod::IcTag as IcTag;

/// The struct defining a single message to and from the server and client.
#[derive(Clone)]
pub struct IcCommand { cmd: Vec<String>,data: Vec<u8> }
impl IcCommand {
	/// Will translate an [`IcPacket`] to an [`IcCommand`]
	pub fn from_packet(p: IcPacket) -> IcCommand {
		let mut proto_iccmd = IcCommand { cmd:Vec::new(),data: p.body.unwrap_or(Vec::new()) };
		proto_iccmd.from_string(p.header.unwrap_or("".to_string()));
		proto_iccmd
	}
	/// Will create a new [`IcCommand`] from `raw_cmd` with an empty data buffer.
	pub fn new(raw_cmd: String) -> IcCommand {
		let mut proto_iccmd = IcCommand { cmd: Vec::new(), data: Vec::new() };
		proto_iccmd.from_string(raw_cmd);
		proto_iccmd
	}

	fn from_string(&mut self, s: String) {
		self.cmd = self.finalize_command(s.split_whitespace().collect::<Vec<&str>>());
	}

	fn finalize_command(&self,cmd: Vec<&str>) -> Vec<String> {
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
	
	/// Will create a new [`IcCommand`] from a formatted `input` with data buffer `d` 
	/// (or an empty buffer if `d` is `None`)
	pub fn from_formated_vec(input: Vec<String>,d: Option<Vec<u8>>) -> IcCommand {
		IcCommand { cmd:input,data: d.unwrap_or(Vec::new()) }
	}

	fn parse(&self) -> Box<dyn IcExecute<Connection = MysqlConnection>> {
		//Returns an ic_execute by parsing cmd
		//0: null
		//1: dir
		//2: unbaked_entry
		//3: tag
		//4: show
		//-1: exit
		if self.cmd.len() <= 0 {return Box::new(IcNull::new())}
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
			return Box::new(IcNull::new());
		} else if return_type == -1 {
			return Box::new(IcNull::new());
		} else if return_type == 1 {
			return Box::new(IcDir::new(self.cmd[1..].to_vec()));
		} else if return_type == 2 {
			return Box::new(IcEntry::from_ic_command(self.clone()));
		} else if return_type == 3 {
			return Box::new(IcTag::new(self.cmd[1..].to_vec()));
		} else if return_type == 4 {
			return Box::new(IcAll::new(self.cmd[1..].to_vec()));
		} else {
			return Box::new(IcNull::new());
		}
	}
	
	/// Parse [`IcCommand`] to [`IcPacket`]
	pub fn to_ic_packet(&self) -> IcPacket {
		let mut s = String::new();
		for t in &self.cmd {
			if t.contains(char::is_whitespace) {
				s.push_str(&("((".to_owned()+&t+")) "));
			}else {s.push_str(&(t.to_owned()+" "))}
		}
		s = s.trim_end().to_string();
		IcPacket::new(Some(s),Some(self.clone().data))
	}

	#[tokio::main]
	async fn handle(cmd_opts: IcCommand) -> IcPacket {
		let mut connection = establish_connection();
		let mut cmd_parsed = cmd_opts.parse();
		cmd_parsed.exec(Some(&mut connection))
	}

}
impl IcExecute for IcCommand {
	type Connection = MysqlConnection;
	
	fn exec(&mut self,_con: Option<&mut Self::Connection>) -> IcPacket {
		IcCommand::handle(self.clone())
	}
}
impl Display for IcCommand {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = String::new();
		for c in &self.cmd {
			s.push_str(&c);
			s.push(' ');
		}
		write!(f,"{}", s)
	}
}

