use std::str;
use std::fmt;
use std::fs;
use std::process;
use std::process::Command;
use std::net::TcpStream;
use std::{thread, time};
use std::fmt::Display;
use std::io::{stdout,stdin,Read,ErrorKind,Error,Write};
use crate::ichandler::ic_types::{ic_connection,ic_packet,ic_execute,ic_response,ic_command};

#[derive(Debug)]
#[derive(PartialEq)]
pub enum ic_client_mode {
	GET, //To local file
	CAT, //To stdout
	SEND,
	EXIT,
	NONE,
}

pub struct ic_input { pub input_str: String,pub fmt_str: Vec<String>, pub pwd: i32, pwdstr: String}
impl ic_input {
	pub fn new() -> ic_input {
		let mut proto_ici = ic_input { input_str: "".to_string(), fmt_str: Vec::new(),pwd: 0,pwdstr: "ROOT".to_string() };
		proto_ici.fmt_str.push(String::new());
		proto_ici
	}
	
	pub fn check_exit(&self) -> bool {
		return if self.fmt_str.len() > 0 && self.fmt_str[0] == "exit" {true} else {false};
	}
	pub fn flush(&mut self) {
		self.input_str = String::new();
		self.fmt_str = Vec::new();//vec!["".to_string();512];
	}
	
	pub fn prompt(&mut self) -> ic_input_command {
		//println!("ic_input#prompt: pwd is at {}",self.pwd);
		print!("{} > ",self.pwdstr);
		stdout().flush();
		stdin().read_line(&mut self.input_str).expect("Error reading line");
		self.input_str = self.input_str.trim_right().to_string();
		let s = &self.input_str;
		ic_input_command::from_input(self)
	}
	
	pub fn set_pwd(&mut self, pwdid: i32,client: &mut ic_client) -> bool {
		if pwdid < 0 {println!("NS1");return false;}
		else if pwdid == 0 {self.pwd = pwdid;self.pwdstr = "ROOT".to_string();return true;}
		let mut p = Vec::<String>::new();
		p.push("DIR".to_string());
		p.push("VALIDATE".to_string());
		p.push(pwdid.to_string());
		let icp = ic_input_command::from_vec(self,p);
		
		//-> DIR VERIFY <DIRID>
		//<- true/false {Dir name/None}
		client.con.send_packet(icp.to_ic_command().to_ic_packet());
		let resp = client.con.get_packet();
		if resp.header.as_ref().unwrap() == "true" {
			self.pwdstr = str::from_utf8(&resp.body.unwrap()).unwrap().to_string();
			self.pwd = pwdid;
			return true;
		} else { return false; }
	}
	
	pub fn write_entry() -> String {
		Command::new("vim").arg("/tmp/tmpentry").status().expect("Failed to open editor");
		let ret = str::from_utf8(&fs::read("/tmp/tmpentry").unwrap()).unwrap().to_string();
		fs::remove_file("/tmp/tmpentry").unwrap();
		ret
	}
	
}

pub struct ic_client { con: ic_connection,pub mode: ic_client_mode }
impl ic_client {
	pub fn connect(ip: &str) -> Result<ic_client,Error> {;
		let con = TcpStream::connect(ip.to_owned()+":64209");
		if let Ok(c) = con {
			return Ok(ic_client { con: ic_connection::new(c),mode: ic_client_mode::CAT });
		} else {
			return Err(Error::new(ErrorKind::Other,"Failed to connect."));
		}
	}
	
	pub fn exec_cmd(&mut self,c: &mut ic_input_command) {
		self.update_mode(c);
		//println!("CLIENT MODE: {:?}",self.mode);
		//println!("SEND IC_PACKET : {}\n{:?}",c.to_ic_command().to_ic_packet().header.unwrap_or("None".to_string()),c.to_ic_command().to_ic_packet().body.unwrap().len());
		let mut sr: ic_packet = ic_packet::new_empty();
		//println!("RECV IC_PACKET : {}\n{:?}",(&sr).header.as_ref().unwrap_or(&"None".to_string()),(&sr).body.as_ref().unwrap_or(&Vec::new()).len());
		if self.mode != ic_client_mode::NONE {
			self.con.send_packet(c.to_ic_command().to_ic_packet()); 
			sr = self.con.get_packet();
		}
		match self.mode {
		ic_client_mode::CAT => {
			println!("{}",std::str::from_utf8(&sr.body.unwrap_or(Vec::new())).unwrap());
		},
		ic_client_mode::GET => {
			fs::write(c.cmd[2].clone(),sr.body.unwrap());
		},
		ic_client_mode::EXIT => {
			process::exit(1);
		},
		ic_client_mode::NONE => {
			if c.cmd[0] == "cd" {
				let res = if c.cmd.len() > 1 {
					c.ref_in.set_pwd(str::parse::<i32>(&c.cmd[1]).unwrap_or(-1),self)
				} else {c.ref_in.set_pwd(0,self)};
				if !res {println!("Ok!.\n")} else {println!("Ok!.\n")};
			} 
		},
		_ => { },
		};
	}

	pub fn update_mode(&mut self,c: &ic_input_command) {
		self.mode = match c.cmd[0].as_ref() {
		"new" | "set" | "mv" | "import" => ic_client_mode::SEND,
		"exit" | "quit" => ic_client_mode::EXIT,
		"cd" => ic_client_mode::NONE,
		"get" => ic_client_mode::GET,
		_ => ic_client_mode::CAT, //rm,ls
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
	pub fn from_vec<'a>(input: &'a mut ic_input,v: Vec<String>) -> ic_input_command<'a> {
		//format the input
		//check for ((tokens That are included between these))
		//If found, concat to one str
		let mut con = false;
		let mut concatenated_str = String::new();
		let mut fcmd = Vec::new();
		if v.len() == 0 {
			return ic_input_command { cmd:Vec::new(), databuff: vec![0;512], ref_in: input}
		}
		for c in v {
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
						concatenated_str.push_str(&c);
					} 
				} else { concatenated_str.push(' '); concatenated_str.push_str(&c) }
			}
		}
		ic_input_command { cmd:fcmd, databuff: vec![0;512],ref_in: input }
	}
	pub fn to_ic_command(&self) -> ic_command {
		let mut fmt_vec:Vec<String> = Vec::new();
		match self.cmd[0].as_ref() {
		"new" | "import" => {
			/*	new [<name>] [UNDER <dir id>]
				CREATE <TYPE> <NAME> <SIZE> UNDER <LOC>"
				<DATA>*/
			fmt_vec.push("ENTRY".to_string());
			fmt_vec.push("CREATE".to_string());
			fmt_vec.push(if self.databuff.len() > 65535 {"ipfs_file".to_string()} else {"text".to_string()});
			if self.cmd[0] == "new" {
				if self.cmd.len() >= 2 {
					fmt_vec.push(self.cmd[1].clone());
					fmt_vec.push(self.databuff.len().to_string());
					fmt_vec.push(if self.cmd.len() > 3 {self.cmd[3].clone()} else {"".to_string()});
					fmt_vec.push(if self.cmd.len() > 3 {"UNDER".to_string()} else {"".to_string()});
				}
			}else if self.cmd[0] == "import" {
					fmt_vec.push(self.cmd[2].clone());
					fmt_vec.push(self.databuff.len().to_string());
			}
			return ic_command::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"get" => {
			/*	GET 11 [<name>]
				ENTRY GET 11 <name>*/
			fmt_vec.push("ENTRY".to_string());
			fmt_vec.push("GET".to_string());
			fmt_vec.push(self.string_wrap(self.cmd[1].clone()));
			fmt_vec.push(self.cmd[2].clone());
			return ic_command::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"ls" => {
			/*	SHOW <Dir id>
				same as 
				DIR SHOW <ID>
				and
				ENTRY SHOW <ID> */
			fmt_vec.push("SHOW".to_string());
			if self.cmd.len() >= 2 {
				fmt_vec.push(self.cmd[1].clone());
			} else {
				fmt_vec.push(self.ref_in.pwd.to_string());
			}
			return ic_command::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"rm" => {
			/*	ENTRY DELETE <Dir id> */
			fmt_vec.push("ENTRY".to_string());
			fmt_vec.push("DELETE".to_string());
			fmt_vec.push(self.cmd[1].clone());
			return ic_command::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"set" => {
			/*	set <eid> <newname>
				ENTRY SET <entry id> <newname> */
			fmt_vec.push("ENTRY".to_string());
			fmt_vec.push("SET".to_string());
			fmt_vec.push(self.cmd[1].clone());
			return ic_command::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"mv" => {
			/*	mv <ID>[/] <newdir>
				DIR MOVE <dirid> <newdirid>
				OR
				ENTRY MOVE <entryid> <dirid>
			*/
			//IF ending with /
			if self.cmd[1].chars().last().unwrap() == '/' {
				fmt_vec.push("DIR".to_string());
				fmt_vec.push("SET".to_string());
				fmt_vec.push(self.cmd[1][..self.cmd[1].len() - 1].to_string());
				fmt_vec.push(self.cmd[2].clone());
			} else {
				fmt_vec.push("ENTRY".to_string());
				fmt_vec.push("SET".to_string());
				fmt_vec.push(self.cmd[1].clone());
				fmt_vec.push(self.cmd[2].clone());
			}
			
			return ic_command::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"mkdir" => {
			/*	mkdir 
				DIR CREATE ((name)) UNDER <DIRID>
			*/
			fmt_vec.push("DIR".to_string());
			fmt_vec.push("CREATE".to_string());
			fmt_vec.push(self.cmd[1].clone());
			fmt_vec.push("UNDER".to_string());
			fmt_vec.push(self.ref_in.pwd.to_string());
			return ic_command::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"rmdir" => {
			/*	rmdir dirid
				DIR DELETE ((name)) UNDER <DIRID>
			*/
			fmt_vec.push("DIR".to_string());
			fmt_vec.push("DELETE".to_string());
			fmt_vec.push(self.cmd[1].clone());
			return ic_command::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"tag" => {
			/*	tag 10[/] <tagid>
			*/
			if self.cmd[1].chars().last().unwrap() == '/' {
				fmt_vec.push("TAG".to_string());
				fmt_vec.push("DIR".to_string());
				fmt_vec.push(self.cmd[1][..self.cmd[1].len() - 1].to_string());
				fmt_vec.push(self.cmd[2].clone());
			} else {
				fmt_vec.push("TAG".to_string());
				fmt_vec.push("ENTRY".to_string());
				fmt_vec.push(self.cmd[1].clone());
				fmt_vec.push(self.cmd[2].clone());
			}
			return ic_command::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"untag" => {
			if self.cmd[1].chars().last().unwrap() == '/' {
				fmt_vec.push("TAG".to_string());
				fmt_vec.push("UNDIR".to_string());
				fmt_vec.push(self.cmd[1][..self.cmd[1].len() - 1].to_string());
				fmt_vec.push(self.cmd[2].clone());
			} else {
				fmt_vec.push("TAG".to_string());
				fmt_vec.push("UNENTRY".to_string());
				fmt_vec.push(self.cmd[1].clone());
				fmt_vec.push(self.cmd[2].clone());
			}
			/*	untag 10[/] <tagid>
			*/
			return ic_command::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"showtags" => {
			/*	showtags	*/
			fmt_vec.push("TAG".to_string());
			fmt_vec.push("SHOW".to_string());
			return ic_command::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"mktag" => {
			/*	showtags	*/
			fmt_vec.push("TAG".to_string());
			fmt_vec.push("CREATE".to_string());
			fmt_vec.push(self.cmd[1].clone());
			return ic_command::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"rmtag" => {
			/*	showtags	*/
			fmt_vec.push("TAG".to_string());
			fmt_vec.push("DELETE".to_string());
			fmt_vec.push(self.cmd[1].clone());
			return ic_command::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"exit" => {
			fmt_vec.push("EXIT".to_string());
			return ic_command::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		}
		_ => return ic_command::from_formated_vec(self.cmd.clone(),None),
		}
		ic_command::from_formated_vec(self.cmd.clone(),Some(self.databuff.clone()))
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
