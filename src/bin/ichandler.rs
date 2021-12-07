extern crate diesel;

use self::models::*;
use diesel::prelude::*;
use std::env;
use IntelCache::*;
use std::fs;
use std::str;
use std::fs::File;
use tar::Archive;
use futures::TryStreamExt;
use futures::executor::block_on;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use std::thread;
use std::net::{TcpStream, SocketAddrV4, Ipv4Addr, TcpListener};
use std::io::{stdout,stdin,Read, Error,Write};
use std::io;
use std::marker::PhantomData;
use std::process::Command;

//STRUCTS
//STRUCT DECLARATIONS
#[derive(Clone)]
pub struct ic_response { pub internal_val: (Option<i32>,Option<Vec<u8>>), }
#[derive(Clone)]
pub struct ic_command { pub cmd: Vec<String>, }
#[derive(Clone)]
pub struct ic_unbaked_entry { cmd: Vec<String>,n: String, t: String,loc: i32 }
pub struct ic_dir { cmd: Vec<String>, }
pub struct ic_tag {cmd: Vec<String>,}
pub struct ic_null {}
//Server
pub struct ic_server {}
//Server side client
pub struct ic_client { con_stream: TcpStream,buffer: Vec<u8>,}
//Client Side connection
pub struct ic_connection { con_stream: TcpStream,con_filebuff: Vec<u8>,con_buff: Vec<u8> }
pub struct ic_input { pub input_str: String,pub fmt_str: Vec<String> }

//STRUCT IMPLS
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
		let mut proto_iccmd = ic_command { cmd: Vec::new() };
		let rret = r.internal_val.1.unwrap();
		proto_iccmd.from_string(str::from_utf8(&rret).unwrap().to_string());
		proto_iccmd
	}
	pub fn new(raw_cmd: String) -> ic_command {
		let mut proto_iccmd = ic_command { cmd: Vec::new() };
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
		ic_command { cmd:input }
	}

	pub fn parse(&self) -> Box<dyn ic_execute> {
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
			let bytes_read = self.con_stream.read(&mut self.buffer).unwrap();
			let cmd = str::from_utf8(&self.buffer[..bytes_read]).unwrap();
			println!("COMMAND READ: {}",cmd);
			let ic_cmd = ic_command::new(cmd.to_string());
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
				let d = c.read(Some(icr.get_size()));
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
impl ic_connection {
	pub fn connect(ip: &str) -> ic_connection {
		ic_connection { con_stream: TcpStream::connect(ip.to_owned()+":64209").expect("could not connect"),con_buff: vec![0;512],con_filebuff: Vec::new() }
	}
	pub fn send(&mut self,icc: ic_command) {
		let mut cc_str = String::new();
		for c in icc.cmd {
			cc_str += &(c + " ");
		}
		cc_str = cc_str.trim_start().to_string();
		self.con_stream.write(cc_str.as_bytes()).unwrap();
	}
	pub fn data_send(&mut self,d: &[u8]) {
		self.con_stream.write(d).unwrap();
	}

	pub fn recieve(&mut self) -> String {
		let br = self.con_stream.read(&mut self.con_buff).unwrap();
		str::from_utf8(&mut self.con_buff[..br]).unwrap().to_string()
	}

	pub fn recieve_data(&mut self,filename: String) {
		let mut filesize = 0;
		while filesize == 0 || self.con_filebuff.len() <= filesize {
			if self.con_filebuff.len() == 0 && filesize == 0{
				//First time setup
				let br = self.recieve().len();
				let mut sstr = String::new();
				let mut sc = 1;
				for b in self.con_buff[..br].to_vec() {
					if b == 10 {break}
					//println!("{}",b);
					sstr.push(b as char);
					sc += 1;
				}
				filesize = sstr.parse::<i32>().unwrap() as usize;
				println!("Getting {} ({} bytes)",filename,filesize);
				for b in self.con_buff[sc..].to_vec(){
					if self.con_filebuff.len() + 1 <= filesize.try_into().unwrap(){
						self.con_filebuff.push(b);
					} else if (self.con_filebuff.len() + 1) as i32 == filesize as i32{
						self.con_filebuff.push(b); 
						//Done, write to file filename
						fs::write(filename,self.con_filebuff.clone());
						println!("File downloaded!");
						return ();
					}
				}
				println!("filedata now is {} out of {}",self.con_filebuff.len(),filesize);
			} else if (self.con_filebuff.len() as i32) < filesize as i32 {
				//Put more into filedata (until fill up)
				println!("FILEDATA now is {} out of {}",self.con_filebuff.len(),filesize);
				let br = self.con_stream.read(&mut self.con_buff).unwrap();
				for b in self.con_buff[..br].to_vec() {
					if self.con_filebuff.len() + 1 <= filesize.try_into().unwrap() {
						self.con_filebuff.push(b);
					}else {println!("{} + 1 == {} ({})",self.con_filebuff.len(),self.con_filebuff.len() + 1,filesize);}
				} 
				println!("filedata now is {} out of {}",self.con_filebuff.len(),filesize);
			} else if (self.con_filebuff.len() as i32) == filesize as i32 {
				//Done, write to file filename
				fs::write(filename,self.con_filebuff.clone());
				println!("File downloaded!");
				return ();
			}
		}
	}
}
impl ic_input {
	pub fn new() -> ic_input {
		let mut proto_ici = ic_input { input_str: "".to_string(), fmt_str: Vec::new() };
		proto_ici.fmt_str.push(String::new());
		proto_ici
	}
	pub fn check_exit(&self) -> bool {
		return if self.fmt_str[0] == "EXIT" {true} else {false};
	}
	pub fn flush(&mut self) {
		self.input_str = String::new();
		self.fmt_str = Vec::new();//vec!["".to_string();512];
	}
	pub fn prompt(&mut self) {
		print!("> ");
		io::stdout().flush();
		stdin().read_line(&mut self.input_str).expect("Error reading line");
		self.input_str = self.input_str.trim_right().to_string();
		self.format_input();
	}
	pub fn is_writemode(&self) -> bool {
		if self.fmt_str[0] == "WRITE".to_string() {true} else {false}
	}
	pub fn is_getmode(&self) -> bool {
		if self.fmt_str[0] == "GET" {true} else {false}
	}
	fn format_input(&mut self) {
		//format the input
		//check for ((tokens That are included between these))
		//If found, concat to one str
		let mut con = false;
		let mut concatenated_str = String::new();
		if self.input_str.split_whitespace().collect::<Vec<&str>>().len() == 0 {
			self.fmt_str = Vec::new();
		}
		for c in self.input_str.split_whitespace() {
			if ! con { 
				if c.len() > 1 {
					if c.chars().nth(0).unwrap() == '\"' && ! (c.chars().nth(c.len()-1).unwrap() == '\"'){ 
						con = true; 
						concatenated_str.push_str(&c[1..].to_string());
					} else {
						self.fmt_str.push(c.to_string()); 
					}
				} else { self.fmt_str.push(c.to_string()) }
			} else { 
				if c.len() > 1 {
					if c.chars().nth(c.len()-1).unwrap() == '\"' {
						concatenated_str.push(' ');
						concatenated_str.push_str(&c[..c.len() - 1]);
						self.fmt_str.push(concatenated_str);
						concatenated_str = String::new();
						con = false 
					}else { 
						concatenated_str.push(' ');
						concatenated_str.push_str(c);
					} 
				} else { concatenated_str.push(' '); concatenated_str.push_str(c) }
			}
		}
	}
	
	pub fn write_entry(&mut self) {
		Command::new("vim").arg("/tmp/tmpentry").status().expect("Failed to open editor");
		self.input_str = str::from_utf8(&fs::read("/tmp/tmpentry").unwrap()).unwrap().to_string();
		fs::remove_file("/tmp/tmpentry").unwrap();
	}
	
	fn string_wrap(&self,s: String) -> String {
		if s.contains(char::is_whitespace) {"((".to_owned()+&s+"))"} else {s}
	}

	pub fn to_ic_command(&self) -> ic_command {
		let mut fmt_vec:Vec<String> = Vec::new();
		match self.fmt_str[0].as_ref() {
		"WRITE" => {
		//WRITE [<name>] [UNDER <dir id>]
		//CREATE <TYPE> <NAME> <SIZE> UNDER <LOC>"
			fmt_vec.push("ENTRY".to_string());
			fmt_vec.push("CREATE".to_string());
			fmt_vec.push(if self.input_str.len() > 65535 {"ipfs_file".to_string()} else {"text".to_string()});
			fmt_vec.push(self.string_wrap(self.fmt_str[1].clone()));
			fmt_vec.push(self.input_str.len().to_string());
			fmt_vec.push(if self.fmt_str.len() > 3 {self.fmt_str[3].clone()} else {"".to_string()});
			fmt_vec.push(if self.fmt_str.len() > 3 {"UNDER".to_string()} else {"".to_string()});
			return ic_command::from_formated_vec(fmt_vec);
		},
		"GET" => {
			fmt_vec.push("ENTRY".to_string());
			fmt_vec.push("GET".to_string());
			fmt_vec.push(self.string_wrap(self.fmt_str[1].clone()));
			fmt_vec.push(self.fmt_str[3].clone());
			return ic_command::from_formated_vec(fmt_vec);
		},
		_ => return ic_command::from_formated_vec(self.fmt_str.clone()),
		}
		ic_command::from_formated_vec(self.fmt_str)
	}
}

//TRAITS
//TRAIT DECLARATION
pub trait ic_execute {
	fn exec(&self,con: Option<&MysqlConnection>) -> ic_response;
}
//ic_execute TRAIT IMPLs
impl ic_execute for ic_command {
	fn exec(&self,con: Option<&MysqlConnection>) -> ic_response {
		
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
	fn exec(&self,con: Option<&MysqlConnection>) -> ic_response {
		let mut create = false;
		let mut delete = false;
		let mut retstr: String = "OK.\n".to_string();
		match self.cmd[0].as_str() {
		"DELETE" => delete = true,
		"SHOW" => retstr = show_dirs(con.unwrap()),
		"CREATE" => create = true,
		_ => eprintln!("{} is not a valid subcommand of DIR",self.cmd[0]),
		}

		if create {
			//CREATE ((NAME))
			if self.cmd.len() == 2 {
				create_dir(con.unwrap(),&self.cmd[1],None);
			} else if ( self.cmd.len() == 4 ) {
				//CREATE ((NAME)) UNDER <DIR ID>
				if self.cmd[2] == "UNDER" {
					create_dir(con.unwrap(),&self.cmd[1],Some(self.cmd[3].parse::<i32>().unwrap()));
				} 
			}
		}
		if delete {
			if self.cmd.len() == 2 {
				delete_dir(con.unwrap(),self.cmd[1].parse::<i32>().unwrap());
			}
		}
		ic_response::from_str(retstr)
	}
}
impl ic_execute for ic_null {
	fn exec(&self,con: Option<&MysqlConnection>) -> ic_response {
		ic_response::null_response()
	}
}
impl ic_execute for ic_unbaked_entry {
	fn exec(&self,con: Option<&MysqlConnection>) -> ic_response {
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
				delete_entry(con.unwrap(),self.cmd[1].parse::<i32>().unwrap());
			}
		}
		if show {
			rstr = show_entries(con.unwrap(),Some(false),Some(true));
			//return (Some(rstr.len() as i32),Some(rstr.as_bytes().to_vec()));
			return ic_response::from_str(rstr);
		}
		if get {
			//GET 1 file.txt
			use models::Entry;
			let e = get_entry_by_id(con.unwrap(),self.cmd[1].parse::<i32>().unwrap());
			
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
	fn exec(&self,con: Option<&MysqlConnection>) -> ic_response {
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
				delete_tag(&con.unwrap(), (&self.cmd[1]).parse::<i32>().unwrap());
			}
		}

		if show {
			rstr = show_tags(&con.unwrap(),Some(true));
			//return (if rstr.len() != 0 {Some(rstr.len() as i32)} else {None},if rstr.len() != 0 {Some(rstr)} else {None});
			return ic_response::from_str(rstr);
		}

		if create {
			//CREATE <TAG>
			if self.cmd.len() == 2 {
				create_tag(&con.unwrap(), &self.cmd[1]);
			}
		}

		if tagdir == 1{
			//DIR <DIRID> <TAGID>
			if self.cmd.len() == 3 {
				tag_dir(&con.unwrap(), (&self.cmd[1]).parse::<i32>().unwrap(),(&self.cmd[2]).parse::<i32>().unwrap());
			}
		} else if tagdir == -1 {
			//UNDIR <DIRID> <TAGID>
			if self.cmd.len() == 3 {
				untag_dir(&con.unwrap(), (&self.cmd[1]).parse::<i32>().unwrap(),(&self.cmd[2]).parse::<i32>().unwrap());
			}
		}

		if tagentry == 1{
			if self.cmd.len() == 3 {
				tag_entry(&con.unwrap(), (&self.cmd[1]).parse::<i32>().unwrap(),(&self.cmd[2]).parse::<i32>().unwrap());
			}
		} else if tagentry == -1 {
			if self.cmd.len() == 3 {
				untag_entry(&con.unwrap(), (&self.cmd[1]).parse::<i32>().unwrap(),(&self.cmd[2]).parse::<i32>().unwrap());
			}
		}
		//(Some(4),Some("OK.\n".to_string()))
		ic_response::from_str(rstr)
	}
}

//HANDLE FNs
pub fn handle_dir(cmd_opts: ic_command) -> ic_response {
	use self::schema::dir::dsl::*;
	let connection = establish_connection();
	let cmd_parsed = cmd_opts.parse();
	cmd_parsed.exec(Some(&connection))
	//str::from_utf8(&cmd_parsed.exec(Some(&connection)).1.unwrap_or("INVALID".as_bytes().to_vec())).unwrap().to_string()
}
#[tokio::main]
pub async fn handle_entry(cmd_opts: ic_command) -> ic_response {
	let connection = establish_connection();
	let cmd_parsed = cmd_opts.parse();
	cmd_parsed.exec(Some(&connection))
}
pub fn handle_tag(cmd_opts: ic_command) -> ic_response {
	let connection = establish_connection();
	let cmd_parsed = cmd_opts.parse();
	cmd_parsed.exec(Some(&connection))
}

//MISC
#[cfg(not(windows))]
const EOF: &str = "CTRL+D";

#[cfg(windows)]
const EOF: &str = "CTRL+Z";
