use std::str;
use std::net::TcpStream;
use std::io::{self,BufRead,BufReader,stdout,stdin,Read,Write};
use std::process::Command;
use std::process;
use std::fs;
use std::{thread, time};
use std::fmt::Display;
use std::fmt;

use super::ic_server::{ic_response,ic_command};
use super::ic_execute;

#[derive(Debug)]
pub enum ic_input_cmd_mode {
	READ,
	GET,
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
		let pwdstr = if self.pwd > 1 {self.pwd.to_string()} else {"ROOT".to_string()};
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

pub struct ic_connection { con_stream: TcpStream,con_filebuff: Vec<u8>,con_buff: Vec<u8> }
impl ic_connection {
	pub fn connect(ip: &str) -> ic_connection {
		ic_connection { con_stream: TcpStream::connect(ip.to_owned()+":64209").expect("could not connect"),con_buff: vec![0;512],con_filebuff: Vec::new() }
	}
	
	pub fn send(&mut self,icc: ic_command) {
		println!("ic_connection#send: sending ({:?})",icc.to_string());
		self.con_stream.write(icc.to_string().as_bytes()).unwrap();
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

pub struct ic_input_command<'a> { pub cmd: Vec<String>, databuff: Vec<u8>,ref_in: &'a mut ic_input }
impl ic_execute for ic_input_command<'_> {
	type Connection = ic_connection;

	fn exec(&mut self,mut con: Option<&mut Self::Connection>) -> ic_response {
		println!("ic_input_command#exec: mode is {:?}",self.get_mode());
		match self.get_mode() {
		ic_input_cmd_mode::READ =>{
			println!("ic_input_command#exec: ic_command is ({:?})",self.to_ic_command().cmd);
			con.as_mut().unwrap().send(self.to_ic_command()); 
			if ! (self.cmd[0] == "EXIT") { 
				let sr = con.as_mut().unwrap().recieve();
				print!("{}",sr);
			};
		},
		ic_input_cmd_mode::WRITE => {
			if self.cmd.len() > 1 {
				self.databuff = ic_input::write_entry().as_bytes().to_vec();
			} else {
				self.cmd.push(String::new());
				println!("Name?");
				let mut n = String::new();
				stdin().read_line(&mut n).unwrap();
				self.cmd[1] = n;
				self.databuff = ic_input::write_entry().as_bytes().to_vec();
			}
			con.as_mut().unwrap().send(self.to_ic_command());
			thread::sleep(time::Duration::from_millis(10));
			con.as_mut().unwrap().data_send(&self.databuff);
		},
		ic_input_cmd_mode::GET => {
			if ! (self.cmd.len() == 4) && self.cmd.len() >= 2{
				println!("File name?");
				self.cmd.push("AS".to_string());
				self.cmd.push(String::new());
				stdin().read_line(&mut self.cmd[3]).unwrap();
				self.cmd[3] = self.cmd[3].trim_end().to_string();
			} else {println!("{} {}",! (self.cmd.len() == 4),self.cmd.len() >= 2)}
			con.as_mut().unwrap().send(self.to_ic_command());
			con.as_mut().unwrap().recieve_data(self.cmd[3].clone()); 
		},
		ic_input_cmd_mode::EXIT => {
			process::exit(1);
		},
		ic_input_cmd_mode::NONE => {
			//Do not send or recieve
			if self.cmd[0] == "cd" {
				if self.cmd.len() > 1 {self.ref_in.pwd = str::parse::<i32>(&self.cmd[1]).unwrap_or(0)} else {self.ref_in.pwd = 0}
				return ic_response::from_str("cd ".to_string()+&self.ref_in.pwd.to_string());
			}
		},
		_ => { eprintln!("ERR: NOT MATCHED: {:?}", self.get_mode()) }
		};
		ic_response::null_response()
	}
}
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
	pub fn is_writemode(&self) -> bool {
		if self.cmd[0] == "WRITE".to_string() {true} else {false}
	}
	pub fn is_getmode(&self) -> bool {
		if self.cmd[0] == "GET" {true} else {false}
	}
	pub fn get_mode(&self) -> ic_input_cmd_mode {
		if ! (self.cmd[0] == "cd") && ! self.is_writemode() && ! self.is_getmode() && self.cmd[0] != "EXIT" {
			return ic_input_cmd_mode::READ;
		} else if ! (self.cmd[0] == "cd") && ! self.is_getmode() && self.cmd[0] != "EXIT" {
			return ic_input_cmd_mode::WRITE;
		} else if ! (self.cmd[0] == "cd") && self.cmd[0] != "EXIT" { 
			return ic_input_cmd_mode::GET;
		} else if self.cmd[0] == "cd" {
			return ic_input_cmd_mode::NONE
		} else {
			return ic_input_cmd_mode::EXIT;
		}
	}
	pub fn to_ic_command(&self) -> ic_command {
		let mut fmt_vec:Vec<String> = Vec::new();
		match self.cmd[0].as_ref() {
		"WRITE" => {
		//WRITE [<name>] [UNDER <dir id>]
		//CREATE <TYPE> <NAME> <SIZE> UNDER <LOC>"
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
		"GET" => {
			//GET 11 [AS <name>]
			//ENTRY GET 11 <name>
			fmt_vec.push("ENTRY".to_string());
			fmt_vec.push("GET".to_string());
			fmt_vec.push(self.string_wrap(self.cmd[1].clone()));
			fmt_vec.push(self.cmd[3].clone());
			return ic_command::from_formated_vec(fmt_vec);
		},
		_ => return ic_command::from_formated_vec(self.cmd.clone()),
		}
		ic_command::from_formated_vec(self.cmd)
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
