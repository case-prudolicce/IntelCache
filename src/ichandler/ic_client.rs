use std::str;
use std::net::TcpStream;
use std::io::{stdout,stdin,Read,Write};
use std::process::Command;
use std::fs;

use crate::ichandler::ic_server::ic_command;


pub struct ic_input { pub input_str: String,pub fmt_str: Vec<String> }
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
		stdout().flush();
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

pub struct ic_connection { con_stream: TcpStream,con_filebuff: Vec<u8>,con_buff: Vec<u8> }
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
