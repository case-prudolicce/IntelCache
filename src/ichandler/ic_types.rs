use std::str;
use std::io::{ErrorKind,Error,self,BufRead,BufReader,stdout,stdin,Read,Write};
use std::process::Command;
use std::process;
use std::fs;
use std::{thread, time};
use std::fmt;
use crate::*;
use futures::TryStreamExt;
use futures::executor::block_on;
use std::fs::File;
use tar::Archive;
use std::net::{TcpStream, SocketAddrV4, Ipv4Addr, TcpListener};
use std::fmt::Display;

use diesel::MysqlConnection;

use crate::{untag_entry,tag_entry,untag_dir,tag_dir,create_tag,show_tags,delete_tag,establish_connection};

#[derive(Clone)]
pub struct ic_response { pub internal_val: (Option<i32>,Option<Vec<u8>>), }
pub trait ic_execute {
	type Connection;
	
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> ic_response;
}
#[derive(Clone)]
pub struct ic_command { pub cmd: Vec<String>,pub data: Vec<u8> }
#[derive(Clone)]
pub struct ic_unbaked_entry { pub cmd: Vec<String>,pub n: String, pub t: String,pub loc: i32 }
pub struct ic_dir { cmd: Vec<String>, }
pub struct ic_all { cmd: Vec<String>, }
pub struct ic_tag {cmd: Vec<String>,}
pub struct ic_null {}

impl ic_all {
	pub fn new(args: Vec<String>) -> ic_all {
		ic_all { cmd: args }
	}
}
impl ic_dir {
	pub fn new(args: Vec<String>) -> ic_dir {
		ic_dir { cmd: args }
	}
}
impl ic_null {
	pub fn new() -> ic_null {
		ic_null { }
	}
}
impl ic_command {
	pub fn from_response(r: ic_response) -> ic_command {
		let mut proto_iccmd = ic_command { cmd: Vec::new(), data: Vec::new() };
		let rret = r.internal_val.1.as_ref().unwrap();
		proto_iccmd.from_string(str::from_utf8(&rret).unwrap().to_string());
		println!("ic_command#from_response: ic_response is ({:?},{:?})",r.internal_val.0,r.internal_val.1);
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
	
	pub fn from_formated_vec(input: Vec<String>) -> ic_command {
		ic_command { cmd:input,data: Vec::new() }
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
			return Box::new(ic_unbaked_entry::new(self.cmd[1..].to_vec()));
		} else if return_type == 3 {
			return Box::new(ic_tag::new(self.cmd[1..].to_vec()));
		} else if return_type == 4 {
			return Box::new(ic_all::new(self.cmd[1..].to_vec()));
		} else {
			return Box::new(ic_null::new());
		}
	}
}
impl ic_unbaked_entry{
	pub fn new(args: Vec<String>) -> ic_unbaked_entry {
		ic_unbaked_entry { cmd: args,n:"".to_string(),t:"".to_string(),loc:0, }
	}
	pub fn from_ic_command(icc: ic_command) -> ic_unbaked_entry {
		println!("ICC @ UNBAKED_ENTRY: {:?}",icc.cmd);
		ic_unbaked_entry { cmd: icc.cmd.clone(),n:icc.cmd[0].to_owned(),t:icc.cmd[1].to_owned(),loc:if icc.cmd.len() > 2 {icc.cmd[2].parse::<i32>().unwrap()} else {1}, }
	}
	pub fn new_empty() -> ic_unbaked_entry {
		ic_unbaked_entry { cmd: Vec::new(),n:"".to_string(),t:"".to_string(),loc:0, }
	}
	pub fn bake(&self,data: &[u8]) {
		println!("Baking {} ({} {}) with data.",self.n,self.t,self.loc);
		let con = establish_connection();
		match self.t.as_ref() {
		"text" => Some(make_text_entry(&con,&self.n,str::from_utf8(data).unwrap(),Some(self.loc),None)),
		"ipfs_file" => Some(block_on(make_file_entry(&con,&self.n,data.to_vec(),Some(self.loc),None))),
		_ => None,
		};
	}
}
impl ic_response {
	//(DATA-SIZE,DATA)
	pub fn from_str(string: String) -> ic_response {
		ic_response { internal_val: (Some(string.len() as i32),Some(string.as_bytes().to_vec())) }
	}
	//(length*-1,data)
	pub fn data_get_response_from_str(string: String,length: i32) -> ic_response {
		ic_response { internal_val: (Some((length) * -1),Some(string.as_bytes().to_vec())) }
	}
	//(None,None)
	pub fn null_response() -> ic_response {
		ic_response { internal_val: (None,None) }
	}
	//(DATA-SIZE,data_size+\n+data)
	pub fn data_response(data: Vec<u8>) -> ic_response {
		//APPEND SIZE TO internal_val.1
		ic_response { internal_val: (Some(data.len() as i32),Some([data.len().to_string().as_bytes(),&[10_u8],&data].concat())) }
	}
	//(DATA-SIZE * -1,data)
	pub fn data_get_response(data: Vec<u8>) -> ic_response {
		ic_response { internal_val: (Some((data.len() as i32) * -1),Some(data)) }
	}
	//(0,None)
	pub fn exit_response() -> ic_response {
		ic_response { internal_val: (Some(0),None) }
	}

	pub fn is_exit(&self) -> bool {
		if self.internal_val.0 != None && self.internal_val.0.unwrap() == 0 {true} else {false}
	}
	pub fn is_getting(&self) -> bool {
		if self.internal_val.0 != None && self.internal_val.0.unwrap() < 0 {true} else {false}
	}
	pub fn is_sending(&self) -> bool {
		if self.internal_val.0 != None && self.internal_val.0.unwrap() > 0 {true} else {false}
	}
	
	pub fn get_size(&self) -> i32 {
		return if self.internal_val.0.unwrap() < 0 {self.internal_val.0.unwrap()*-1} else {self.internal_val.0.unwrap()}
	}
}
impl ic_tag {
	pub fn new(args: Vec<String>) -> ic_tag {
		ic_tag { cmd: args }
	}
}

impl ic_execute for ic_all {
	type Connection = MysqlConnection;
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> ic_response {
		let mut retstr: String = "OK.\n".to_string();
		println!("ic_all#exec: cmd looks like {:?}",self.cmd);
		if self.cmd.len() == 1 {
			retstr = show_dirs(con.as_ref().unwrap(),Some(self.cmd[0].parse::<i32>().unwrap()));
			retstr += &show_entries(con.as_ref().unwrap(),Some(false),Some(true));
		} else {
			retstr = show_dirs(con.as_ref().unwrap(),None);
			retstr += &show_entries(con.as_ref().unwrap(),Some(false),Some(true));
		}
		ic_response::from_str(retstr)
	}
}
impl ic_execute for ic_command {
	type Connection = MysqlConnection;
	
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> ic_response {
		handle(self.clone())
	}
}
impl ic_execute for ic_dir {
	type Connection = MysqlConnection;
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> ic_response {
		let mut create = false;
		let mut delete = false;
		let mut show = false;
		let mut retstr: String = "OK.\n".to_string();
		match self.cmd[0].as_str() {
		"DELETE" => delete = true,
		"SHOW" => show = true,
		"CREATE" => create = true,
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
		ic_response::from_str(retstr)
	}
}
impl ic_execute for ic_null {
	type Connection = MysqlConnection;
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> ic_response {
		ic_response::null_response()
	}
}
impl ic_execute for ic_unbaked_entry {
	type Connection = MysqlConnection;
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> ic_response {
		let mut get = false;
		let mut create = false;
		let mut delete = false;
		let mut show = false;
		let mut rstr = "OK.\n".to_string();

		match self.cmd[0].as_str() {
		"DELETE" => delete = true,
		"SHOW" => show = true,
		"CREATE" => create = true,
		"GET" => get = true,
		_ => eprintln!("{} is not a valid subcommand of ENTRY",self.cmd[0]),
		}
		
		if create {
			//"CREATE <TYPE> <NAME> <SIZE> UNDER <LOC>"
			let mut retstr = String::new();
			if self.cmd.len() >= 4 {
				if self.cmd[2].contains(char::is_whitespace) {
					retstr.push('(');
					retstr.push('(');
					retstr.push_str(&self.cmd[2]);
					retstr.push(')');
					retstr.push(')');
					retstr.push(' ');
					retstr.push_str(&self.cmd[1]);
				} else { 
					retstr.push_str(&(self.cmd[2].to_owned()+" "+&self.cmd[1]));
				}
				if self.cmd.len() == 6 && self.cmd[4] == "UNDER" {
					retstr.push_str(&(" ".to_owned()+&self.cmd[5]));
				}
				//return (Some((&self.cmd[3]).to_string().parse::<i32>().unwrap()*-1),Some(retstr.as_bytes().to_vec()));
				return ic_response::data_get_response_from_str(retstr,self.cmd[3].parse::<i32>().unwrap())
			}
		}
		if delete {
			//"DELETE <ID>"
			if self.cmd.len() == 2 {
				delete_entry(con.as_ref().unwrap(),self.cmd[1].parse::<i32>().unwrap());
			}
		}
		if show {
			rstr = show_entries(con.as_ref().unwrap(),Some(false),Some(true));
			//return (Some(rstr.len() as i32),Some(rstr.as_bytes().to_vec()));
			return ic_response::from_str(rstr);
		}
		if get {
			//GET 1 file.txt
			use models::Entry;
			let e = get_entry_by_id(con.as_ref().unwrap(),self.cmd[1].parse::<i32>().unwrap());
			
			if self.cmd.len() == 3 {
				if e.type_ == "ipfs_file" {
					let client = IpfsClient::default();
					match block_on(client
					    .get(str::from_utf8(&e.data).unwrap())
					    .map_ok(|chunk| chunk.to_vec())
					    .try_concat())
					{
					    Ok(res) => {
						fs::write(&self.cmd[2],res).unwrap();

					    }
					    Err(e) => eprintln!("error getting file: {}", e)
					}
					let mut archive = Archive::new(File::open(&self.cmd[2]).unwrap());
					archive.unpack(".").unwrap();
					fs::rename(str::from_utf8(&e.data).unwrap(),&self.cmd[2]).unwrap();
					let ret = fs::read(&self.cmd[2]).unwrap();
					fs::remove_file(&self.cmd[2]).unwrap();
					//return (Some(ret.len() as i32),Some([ret.len().to_string().as_bytes(),&[10_u8],&ret].concat()))
					return ic_response::data_response(ret);
					
				}else if e.type_ == "text" {
					//return (Some(e.data.len() as i32),Some([e.data.len().to_string().as_bytes(),&[10_u8],&e.data].concat()));
					return ic_response::data_response(e.data);
				}
			}
		}
		ic_response::from_str(rstr)
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
pub async fn handle(cmd_opts: ic_command) -> ic_response {
	let mut connection = establish_connection();
	let mut cmd_parsed = cmd_opts.parse();
	cmd_parsed.exec(Some(&mut connection))
}
