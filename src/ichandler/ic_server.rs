use std::str;
use crate::*;
use std::thread;
use futures::TryStreamExt;
use futures::executor::block_on;
use std::fs;
use std::fs::File;
use tar::Archive;
use std::net::{TcpStream, SocketAddrV4, Ipv4Addr, TcpListener};
use std::io::{stdout,stdin,Read, Error,Write};
use std::fmt::Display;
use std::fmt;

use super::ic_execute;

#[derive(Clone)]
pub struct ic_response { pub internal_val: (Option<i32>,Option<Vec<u8>>), }
#[derive(Clone)]
pub struct ic_command { pub cmd: Vec<String>,pub data: Vec<u8> }
#[derive(Clone)]
pub struct ic_unbaked_entry { cmd: Vec<String>,n: String, t: String,loc: i32 }
pub struct ic_dir { cmd: Vec<String>, }
pub struct ic_tag {cmd: Vec<String>,}
pub struct ic_null {}
//Server
pub struct ic_server {}
//Server side client
pub struct ic_client { con_stream: TcpStream,buffer: Vec<u8>,}
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
		//-1: exit
		let mut return_type = 0;
		match self.cmd[0].as_str() {
		"DIR" => return_type = 1,
		"ENTRY" => return_type = 2,
		"TAG" => return_type = 3,
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
impl ic_client {
	pub fn new(c: TcpStream) -> ic_client {
		ic_client { con_stream: c,buffer: [0;512].to_vec() }
	}

	//Will Read whole buffer is bytes_to_read isn't specified.
	pub fn read(&mut self,bytes_to_read: Option<i32>) -> ic_response {
		if bytes_to_read != None {
			let mut databuf: Vec<u8> = Vec::new();
			let mut bytes_read: usize;
			while (databuf.len() as i32) < bytes_to_read.unwrap() { 
				bytes_read = self.con_stream.read(&mut self.buffer).unwrap();
				for b in 0..bytes_read {
					if databuf.len() as i32 + 1 <= bytes_to_read.unwrap() { databuf.push(self.buffer[b]); }
				}
				
				if (databuf.len() as i32) < bytes_to_read.unwrap() { 
					println!("Missing {} bytes",bytes_to_read.unwrap() - databuf.len() as i32); 
				} else if (databuf.len()) as i32 == bytes_to_read.unwrap() { 
					return ic_response::data_response(databuf);
				}
			} 
			println!("All {} Bytes recieved!\n{:?}",bytes_to_read.unwrap(),databuf);
			return ic_response::data_response(databuf);
		} else { 
			//EXIT IF bytes_read = 0
			let bytes_read = self.con_stream.read(&mut self.buffer).unwrap();
			let cmd = str::from_utf8(&self.buffer[..bytes_read]).unwrap();
			println!("COMMAND READ: {}",cmd);
			let mut ic_cmd = ic_command::new(cmd.to_string());
			ic_cmd.exec(None) 
		}
	}

	pub fn write(&mut self,d: &[u8]) {
		self.con_stream.write(d).unwrap();
	}

	pub fn addr(&self) -> String {
		self.con_stream.peer_addr().unwrap().to_string()
	}
}
impl ic_server {
	pub fn handle_client(&self,mut c: ic_client) -> Result<(),Error> {
		println!("Connection received! {:?} is sending data.", c.addr());
		let mut entry: ic_unbaked_entry = ic_unbaked_entry::new_empty();
		loop {
			let icr = c.read(None);
			if icr.is_exit() {
				println!("{:?} is disconnected.", c.addr());
				return Ok(())
			} else if icr.is_getting() {
				println!("Expecting {} bytes from {}",icr.get_size(),c.addr());
				entry = ic_unbaked_entry::from_ic_command(ic_command::from_response(icr.clone()));
				println!("ic_server#handle_client: halfbaked entry is ready ((cmd: {:?},type: {},name: {},loc: {})",entry.cmd,entry.t,entry.n,entry.loc);
				let d = c.read(Some(icr.get_size()));
				println!("ic_server#handle_client: Bytes recieved({:?})",d.internal_val.1.as_ref().unwrap());
				println!("Got bytes,baking....");
				entry.bake(&d.internal_val.1.unwrap());
			} else if icr.is_sending() {
				println!("Sending {} bytes to {}",icr.internal_val.0.unwrap(),c.addr());
				c.write(&icr.internal_val.1.unwrap());
			}
			
		}
	}

	pub fn listen(&'static self) {
		let loopback:Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
		let socket:SocketAddrV4 = SocketAddrV4::new(loopback, 64209);
		let listener = TcpListener::bind(socket).unwrap();
		let port = listener.local_addr().unwrap();
		println!("Listening on {}", port);
		for stream in listener.incoming() { 
			match stream {
				Err(e) => { eprintln!("failed: {}",e) },
				Ok(stream) => { thread::spawn(  move || {
						self.handle_client(ic_client::new(stream)).unwrap_or_else(|error| eprintln!("{:?}",error));
					});
				},
			}
		}
	}
}

//TRAITS
//ic_execute TRAIT IMPLs
impl ic_execute for ic_command {
	type Connection = MysqlConnection;
	
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> ic_response {
		
		let mut DirEntry = 0; //Dir = 1, Entry = -1
		let mut tagging = false;
	
		if self.cmd.len() < 1 { return ic_response::null_response() };
		match self.cmd[0].as_str() {
		"DIR" => DirEntry = 1,
		"ENTRY" => DirEntry = -1,
		"TAG" => tagging = true,
		"EXIT" => return ic_response::exit_response(),
		_ => eprintln!("Invalid."),
		}
		
		let retsize: Option<i32>;
		let retdata: Vec<u8>;
		if ! tagging {
			//Dir handling
			if DirEntry == 1 {
				return handle_dir(self.clone());
			} else if DirEntry == -1 {
			//Entry handling
				return handle_entry(self.clone());
			}
		}else {
			return handle_tag(self.clone());
			//return (if r.0 != None {Some(r.0.unwrap())} else {None},if r.1 != None {Some(r.1.unwrap().as_bytes().to_vec())} else {None})
			
		}
		ic_response::null_response()
	}
}
impl ic_execute for ic_dir {
	type Connection = MysqlConnection;
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> ic_response {
		let mut create = false;
		let mut delete = false;
		let mut retstr: String = "OK.\n".to_string();
		match self.cmd[0].as_str() {
		"DELETE" => delete = true,
		"SHOW" => retstr = show_dirs(con.as_ref().unwrap()),
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
		_ => eprintln!("{} is not a valid subcommand of TAG.",self.cmd[0]),
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
		println!("ic_command#to_string: cmd is ({:?})",self.cmd);
		for c in &self.cmd {
			s.push_str(&c);
			s.push(' ');
		}
		write!(f,"{}", s)
	}
}

pub fn handle_dir(cmd_opts: ic_command) -> ic_response {
	use self::schema::dir::dsl::*;
	let mut connection = establish_connection();
	let mut cmd_parsed = cmd_opts.parse();
	cmd_parsed.exec(Some(&mut connection))
	//str::from_utf8(&cmd_parsed.exec(Some(&connection)).1.unwrap_or("INVALID".as_bytes().to_vec())).unwrap().to_string()
}
#[tokio::main]
pub async fn handle_entry(cmd_opts: ic_command) -> ic_response {
	let mut connection = establish_connection();
	let mut cmd_parsed = cmd_opts.parse();
	cmd_parsed.exec(Some(&mut connection))
}
pub fn handle_tag(cmd_opts: ic_command) -> ic_response {
	let mut connection = establish_connection();
	let mut cmd_parsed = cmd_opts.parse();
	cmd_parsed.exec(Some(&mut connection))
}

