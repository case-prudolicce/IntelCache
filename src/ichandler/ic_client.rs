use std::str;
use std::fmt;
use std::fs;
use std::process;
use std::process::Command;
use std::net::TcpStream;
use std::{thread, time};
use std::fmt::Display;
use std::io::{stdout,stdin,Read,ErrorKind,Error,Write};
use crate::ichandler::ic_types::{ic_execute::ic_execute,ic_response::ic_response,ic_command::ic_command};

#[derive(Debug)]
#[derive(PartialEq)]
pub enum ic_client_mode {
	READ,
	PREPARE_GET,
	GET,
	PREPARE_WRITE,
	WRITE,
	EXIT,
	NONE,
}

pub struct ic_input { pub input_str: String,pub fmt_str: Vec<String>, pub pwd: i32 }
impl ic_input {
	pub fn new() -> ic_input {
		let mut proto_ici = ic_input { input_str: "".to_string(), fmt_str: Vec::new(),pwd: 0 };
		proto_ici.fmt_str.push(String::new());
		proto_ici
	}
	
	pub fn check_exit(&self) -> bool {
		return if self.fmt_str.len() > 0 && self.fmt_str[0] == "EXIT" {true} else {false};
	}
	pub fn flush(&mut self) {
		self.input_str = String::new();
		self.fmt_str = Vec::new();//vec!["".to_string();512];
	}
	
	pub fn prompt(&mut self) -> ic_input_command {
		//println!("ic_input#prompt: pwd is at {}",self.pwd);
		let pwdstr = if self.pwd >= 1 {self.pwd.to_string()} else {"ROOT".to_string()};
		print!("{} > ",pwdstr);
		stdout().flush();
		stdin().read_line(&mut self.input_str).expect("Error reading line");
		self.input_str = self.input_str.trim_right().to_string();
		let s = &self.input_str;
		ic_input_command::from_input(self)
	}
	
	pub fn write_entry() -> String {
		Command::new("vim").arg("/tmp/tmpentry").status().expect("Failed to open editor");
		let ret = str::from_utf8(&fs::read("/tmp/tmpentry").unwrap()).unwrap().to_string();
		fs::remove_file("/tmp/tmpentry").unwrap();
		ret
	}
	
}

pub struct ic_client { con_stream: TcpStream,con_filebuff: Vec<u8>,con_buff: Vec<u8>,pub mode: ic_client_mode }
impl ic_client {
	pub fn connect(ip: &str) -> Result<ic_client,Error> {
		let con = TcpStream::connect(ip.to_owned()+":64209");
		if let Ok(c) = con {
			return Ok(ic_client { con_stream: c,con_buff: vec![0;512],con_filebuff: Vec::new(),mode: ic_client_mode::READ });
		} else {
			return Err(Error::new(ErrorKind::Other,"Failed to connect."));
		}
	}
	
	pub fn send(&mut self,icc: ic_command) {
		//println!("ic_client#send: sending ({:?})",icc.to_string());
		self.con_stream.write(icc.to_string().as_bytes()).unwrap();
	}
	pub fn data_send(&mut self,d: &[u8]) {
		self.con_stream.write(d).unwrap();
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

	pub fn recieve(&mut self) -> String {
		let br = self.con_stream.read(&mut self.con_buff).unwrap();
		str::from_utf8(&mut self.con_buff[..br]).unwrap().to_string()
	}
	
	pub fn exec_cmd(&mut self,c: &mut ic_input_command) {
		if (self.mode != ic_client_mode::WRITE) && (self.mode != ic_client_mode::GET) {self.update_mode(c)};
		match self.mode {
		ic_client_mode::READ =>{
			self.send(c.to_ic_command()); 
			let sr = self.recieve();
			print!("{}",sr);
		},
		ic_client_mode::PREPARE_WRITE => {
			self.send(c.to_ic_command());
			self.mode = ic_client_mode::WRITE;
		},
		ic_client_mode::WRITE => {
			self.data_send(&c.databuff);
		},
		ic_client_mode::PREPARE_GET => {
			self.send(c.to_ic_command());
			self.mode = ic_client_mode::GET;
		},
		ic_client_mode::GET => {
			self.recieve_data(c.cmd[3].clone()); 
		},
		ic_client_mode::EXIT => {
			process::exit(1);
		},
		ic_client_mode::NONE => {
			if c.cmd[0] == "cd" {
				if c.cmd.len() > 1 {c.ref_in.pwd = str::parse::<i32>(&c.cmd[1]).unwrap_or(0)} else {c.ref_in.pwd = 0;}
			} /*else { 
				println!("ic_input_command#exec: @ NONE; no matches for {}",self.c.cmd[0]); 
			}*/
		},
		};
	}

	pub fn update_mode(&mut self,c: &ic_input_command) {
		self.mode = match c.cmd[0].as_ref() {
		"new" => ic_client_mode::PREPARE_WRITE,
		"exit" | "quit" => ic_client_mode::EXIT,
		"cd" => ic_client_mode::NONE,
		"get" => ic_client_mode::PREPARE_GET,
		_ => ic_client_mode::READ
		}
	}
}

pub struct ic_input_command<'a> { pub cmd: Vec<String>, pub databuff: Vec<u8>,pub ref_in: &'a mut ic_input }
impl ic_input_command<'_> {
	pub fn from_input(input: &mut ic_input) -> ic_input_command {
		//format the input
		//check for ((tokens That are included between these))
		//If found, concat to one str
		let mut con = false;
		let mut concatenated_str = String::new();
		let mut fcmd = Vec::new();
		if input.input_str.split_whitespace().collect::<Vec<&str>>().len() == 0 {
			return ic_input_command { cmd:Vec::new(), databuff: vec![0;512], ref_in: input}
		}
		for c in input.input_str.split_whitespace() {
			if ! con { 
				if c.len() > 1 {
					if c.chars().nth(0).unwrap() == '\"' && ! (c.chars().nth(c.len()-1).unwrap() == '\"'){ 
						con = true; 
						concatenated_str.push_str(&c[1..].to_string());
					} else {
						fcmd.push(c.to_string()); 
					}
				} else { fcmd.push(c.to_string()) }
			} else { 
				if c.len() > 1 {
					if c.chars().nth(c.len()-1).unwrap() == '\"' {
						concatenated_str.push(' ');
						concatenated_str.push_str(&c[..c.len() - 1]);
						fcmd.push(concatenated_str);
						concatenated_str = String::new();
						con = false 
					}else { 
						concatenated_str.push(' ');
						concatenated_str.push_str(c);
					} 
				} else { concatenated_str.push(' '); concatenated_str.push_str(c) }
			}
		}
		ic_input_command { cmd:fcmd, databuff: vec![0;512],ref_in: input }
	}
	pub fn to_ic_command(&self) -> ic_command {
		let mut fmt_vec:Vec<String> = Vec::new();
		match self.cmd[0].as_ref() {
		"new" => {
			/*	WRITE [<name>] [UNDER <dir id>]
				CREATE <TYPE> <NAME> <SIZE> UNDER <LOC>"*/
			fmt_vec.push("ENTRY".to_string());
			fmt_vec.push("CREATE".to_string());
			fmt_vec.push(if self.databuff.len() > 65535 {"ipfs_file".to_string()} else {"text".to_string()});
			if self.cmd.len() >= 2 {
				fmt_vec.push(self.string_wrap(self.cmd[1].clone()));
				fmt_vec.push(self.databuff.len().to_string());
				fmt_vec.push(if self.cmd.len() > 3 {self.cmd[3].clone()} else {"".to_string()});
				fmt_vec.push(if self.cmd.len() > 3 {"UNDER".to_string()} else {"".to_string()});
			}
			return ic_command::from_formated_vec(fmt_vec);
		},
		"get" => {
			/*	GET 11 [AS <name>]
				ENTRY GET 11 <name>*/
			fmt_vec.push("ENTRY".to_string());
			fmt_vec.push("GET".to_string());
			fmt_vec.push(self.string_wrap(self.cmd[1].clone()));
			fmt_vec.push(self.cmd[3].clone());
			return ic_command::from_formated_vec(fmt_vec);
		},
		"ls" => {
			/*	SHOW <Dir id>
				same as 
				DIR SHOW <ID>
				and
				ENTRY SHOW <ID> */
			fmt_vec.push("SHOW".to_string());
			fmt_vec.push(self.ref_in.pwd.to_string());
			return ic_command::from_formated_vec(fmt_vec);
		},
		_ => return ic_command::from_formated_vec(self.cmd.clone()),
		}
		ic_command::from_formated_vec(self.cmd.clone())
	}

	fn string_wrap(&self,s: String) -> String {
		if s.contains(char::is_whitespace) {"((".to_owned()+&s+"))"} else {s}
	}
}
impl Display for ic_input_command<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = String::new();
		for c in &self.cmd {
			if c.contains(char::is_whitespace) {
				s.push_str(&("((".to_owned()+&c+"))"));
			}else {
				s.push_str(&c);
			}
			s.push(' ');
		}
		write!(f,"{}", s)
	}
}
