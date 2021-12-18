use std::str;
use std::fs;
use intel_cache_lib::ichandler::ic_client::*;
use intel_cache_lib::ichandler::ic_types::{IcCommand,IcPacket};
use std::process;
use std::io::{stdin};
use std::process::Command;
use std::fmt;
use std::net::TcpStream;
use std::fmt::Display;
use std::io::{stdout,ErrorKind,Error,Write};
use std::env;
pub fn write_entry() -> String {
	let editor = env::var("EDITOR").expect("No Editor env found.");
	Command::new(editor).arg("/tmp/tmpentry").status().expect("Failed to open editor");
	let ret = str::from_utf8(&fs::read("/tmp/tmpentry").unwrap()).unwrap().to_string();
	fs::remove_file("/tmp/tmpentry").unwrap();
	ret
}

fn main() {
	let mut client = IcClient::connect("127.0.0.1").unwrap_or_else(|_| {println!("Failed to connect");process::exit(1)});
	let mut input = IcInput::new();

	loop {
		input.flush();
		let mut input_cmd = input.prompt();
		if input_cmd.cmd.len() <= 0 {continue};
		//Sanity checking/Getting input
		match input_cmd.cmd[0].as_ref() {
		"new" => {
			if input_cmd.cmd.len() > 1 {
				input_cmd.databuff = write_entry().as_bytes().to_vec();
			} else {
				input_cmd.cmd.push(String::new());
				println!("Name?");
				let mut n = String::new();
				stdin().read_line(&mut n).unwrap();
				input_cmd.cmd[1] = n;
				input_cmd.databuff = write_entry().as_bytes().to_vec();
			}
		},
		"import" => {
			if input_cmd.cmd.len() > 2 {
				let dtb = fs::read(&input_cmd.cmd[1]);
				match dtb {
				Ok(data) => input_cmd.databuff = data,
				Err(_e) => {println!("{} is an invalid filename.",&input_cmd.cmd[1]);continue},
				}
			} else if input_cmd.cmd.len() == 2 {
				input_cmd.cmd.push(String::new());
				println!("Name?");
				let mut n = String::new();
				stdin().read_line(&mut n).unwrap();
				input_cmd.cmd[2] = n.trim_end().to_string();
				let dtb = fs::read(&input_cmd.cmd[1]);
				match dtb {
				Ok(data) => input_cmd.databuff = data,
				Err(_e) => {println!("{} is an invalid filename.",&input_cmd.cmd[1]);continue},
				}
			} else {
				input_cmd.cmd.push(String::new());
				input_cmd.cmd.push(String::new());
				let mut p = String::new();
				let mut n = String::new();
				println!("Path?");
				stdin().read_line(&mut p).unwrap();
				println!("Name?");
				stdin().read_line(&mut n).unwrap();
				input_cmd.cmd[1] = p.trim_end().to_string();
				input_cmd.cmd[2] = n.trim_end().to_string();
				let dtb = fs::read(&input_cmd.cmd[1]);
				match dtb {
				Ok(data) => input_cmd.databuff = data,
				Err(_e) => {println!("{} is an invalid filename.",&input_cmd.cmd[1]);continue},
				}
			}
		},
		"get" => { 
			if input_cmd.cmd.len() == 2 {
				if input_cmd.cmd[1].parse::<i32>().unwrap_or(-1) == -1 { println!("{} is an invalid entry id.",input_cmd.cmd[1]);continue; }
				
				input_cmd.cmd.push(String::new());
				println!("Name?");
				let mut n = String::new();
				stdin().read_line(&mut n).unwrap();
				input_cmd.cmd[2] = n.trim_end().to_string();
			}
		},
		"edit" => {
			if input_cmd.cmd.len() == 2 {
				
				if input_cmd.cmd[1].parse::<i32>().unwrap_or(-1) != -1 {
					input_cmd.cmd[0] = "get".to_string();
					input_cmd.cmd.push("/tmp/tmpentry".to_string());
					client.send_cmd(&mut input_cmd.to_ic_command());
					input_cmd.databuff = write_entry().as_bytes().to_vec();
					input_cmd.cmd[0] = "set".to_string();
				} else { println!("{} is an invalid entry id.",input_cmd.cmd[1]);continue; } 
			}
		},
		"rm" | "rmdir" | "rmtag" => { 
			if input_cmd.cmd.len() >= 2 {
				if input_cmd.cmd[1].parse::<i32>().unwrap_or(-1) == -1
				{
					let idtype = match input_cmd.cmd[0].as_ref() {
						"rm" => "entry",
						"rmdir" => "directory",
						"rmtag" => "tag",
						_ => "",
					};
					println!("{} is an invalid {} id.",input_cmd.cmd[1],idtype);
					continue;
				}
			} else {
				let idtype = match input_cmd.cmd[0].as_ref() {
					"rm" => "n entry",
					"rmdir" => " directory",
					"rmtag" => " tag",
					_ => "",
				};
				println!("{} requires a{} id.",input_cmd.cmd[0],idtype);
				continue;
			}
		},
		"tag" | "untag" => {
			if input_cmd.cmd.len() >= 3 {
				if input_cmd.cmd[1].len() == 1 {
					if input_cmd.cmd[1].parse::<i32>().unwrap_or(-1) == -1 {
						println!("{} is an invalid id.",input_cmd.cmd[1]);
						continue;
					} 
				} else {
					//Check last character for a / at the end
					if input_cmd.cmd[1][..input_cmd.cmd[1].len() - 1].parse::<i32>().unwrap_or(-1) == -1 || (input_cmd.cmd[1].parse::<i32>().unwrap_or(-1) == -1 && &input_cmd.cmd[1][input_cmd.cmd[1].len() - 1..] != "/"){ 
						println!("{} is an invalid id.",input_cmd.cmd[1]);
						continue;
					} 
				}

				if input_cmd.cmd[2].parse::<i32>().unwrap_or(-1) == -1
				{
					println!("{} is an invalid tag id.",input_cmd.cmd[2]);
					continue;
				}

			} else {
				println!("{} requires a directory/entry id and a tag id.",input_cmd.cmd[0]);
				continue;
			}
		},
		_ => {}
		};
		//Do something with response 
		let r = client.send_cmd(&mut input_cmd.to_ic_command());
		match input_cmd.cmd[0].as_ref() {
		"ls" | "showtags" => {input.display(r);},
		"mktag" | "rmtag" | "rm" | "rmdir" => {input.resp(r)},
		"exit" | "quit" => {process::exit(1);},
		_ => (),
		};
	}
}
pub struct IcInputCommand<'a> { 
	pub cmd: Vec<String>, 
	pub databuff: Vec<u8>,
	ref_in: &'a mut IcInput 
}
impl IcInputCommand<'_> {
	pub fn from_input(input: &mut IcInput) -> IcInputCommand {
		let mut con = false;
		let mut concatenated_str = String::new();
		let mut fcmd = Vec::new();
		if input.input_str.split_whitespace().collect::<Vec<&str>>().len() == 0 {
			return IcInputCommand { cmd:Vec::new(), databuff: vec![0;512], ref_in: input}
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
		IcInputCommand { cmd:fcmd, databuff: vec![0;512],ref_in: input }
	}
	pub fn from_vec<'a>(input: &'a mut IcInput,v: Vec<String>) -> IcInputCommand<'a> {
		let mut con = false;
		let mut concatenated_str = String::new();
		let mut fcmd = Vec::new();
		if v.len() == 0 {
			return IcInputCommand { cmd:Vec::new(), databuff: vec![0;512], ref_in: input}
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
		IcInputCommand { cmd:fcmd, databuff: vec![0;512],ref_in: input }
	}
	pub fn to_ic_command(&self) -> IcCommand {
		let mut fmt_vec:Vec<String> = Vec::new();
		match self.cmd[0].as_ref() {
		"new" | "import" => {
			fmt_vec.push("ENTRY".to_string());
			fmt_vec.push("CREATE".to_string());
			fmt_vec.push(if self.databuff.len() > 65535 {"ipfs_file".to_string()} else {"text".to_string()});
			if self.cmd[0] == "new" {
				if self.cmd.len() >= 2 {
					fmt_vec.push(self.cmd[1].clone());
					fmt_vec.push(self.databuff.len().to_string());
					fmt_vec.push(if self.cmd.len() > 3 {self.cmd[3].clone()} else {"".to_string()});
					fmt_vec.push("UNDER".to_string());
					fmt_vec.push(if self.cmd.len() == 5 {self.cmd[4].clone()} else {if self.ref_in.pwd != 0 {self.ref_in.pwd.to_string()} else {1.to_string()}});
				}
			}else if self.cmd[0] == "import" {
					fmt_vec.push(self.cmd[2].clone());
					fmt_vec.push(self.databuff.len().to_string());
			}
			return IcCommand::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"get" => {
			fmt_vec.push("ENTRY".to_string());
			fmt_vec.push("GET".to_string());
			fmt_vec.push(self.string_wrap(self.cmd[1].clone()));
			fmt_vec.push(self.cmd[2].clone());
			return IcCommand::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"ls" => {
			if self.cmd.len() >= 2 {
				if self.cmd[1].parse::<i32>().unwrap_or(-1) == -1 {
					match self.cmd[1].chars().nth(0).unwrap() {
					'f' => {
						fmt_vec.push("ENTRY".to_string());
						fmt_vec.push("SHOW".to_string());
						fmt_vec.push(self.cmd[1][1..].to_string());
					},
					'd' => {
						fmt_vec.push("DIR".to_string());
						fmt_vec.push("SHOW".to_string());
						fmt_vec.push(self.cmd[1][1..].to_string());
					}
					'a' => {
						fmt_vec.push("SHOW".to_string());
					}
					_ =>(),
					};
				} else { fmt_vec.push("SHOW".to_string());fmt_vec.push(self.cmd[1].clone()) }
			} else {
				fmt_vec.push("SHOW".to_string());
				fmt_vec.push(self.ref_in.pwd.to_string());
			}
			return IcCommand::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"rm" => {
			fmt_vec.push("ENTRY".to_string());
			fmt_vec.push("DELETE".to_string());
			fmt_vec.push(self.cmd[1].clone());
			return IcCommand::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"set" => {
			fmt_vec.push("ENTRY".to_string());
			fmt_vec.push("SET".to_string());
			fmt_vec.push(self.cmd[1].clone());
			return IcCommand::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"mv" => {
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
			
			return IcCommand::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"mkdir" => {
			fmt_vec.push("DIR".to_string());
			fmt_vec.push("CREATE".to_string());
			fmt_vec.push(self.cmd[1].clone());
			fmt_vec.push("UNDER".to_string());
			fmt_vec.push(self.ref_in.pwd.to_string());
			return IcCommand::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"rmdir" => {
			fmt_vec.push("DIR".to_string());
			fmt_vec.push("DELETE".to_string());
			fmt_vec.push(self.cmd[1].clone());
			return IcCommand::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"tag" => {
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
			return IcCommand::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
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
			return IcCommand::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"showtags" => {
			fmt_vec.push("TAG".to_string());
			fmt_vec.push("SHOW".to_string());
			return IcCommand::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"mktag" => {
			fmt_vec.push("TAG".to_string());
			fmt_vec.push("CREATE".to_string());
			fmt_vec.push(self.cmd[1].clone());
			return IcCommand::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"rmtag" => {
			fmt_vec.push("TAG".to_string());
			fmt_vec.push("DELETE".to_string());
			fmt_vec.push(self.cmd[1].clone());
			return IcCommand::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"exit" => {
			fmt_vec.push("EXIT".to_string());
			return IcCommand::from_formated_vec(fmt_vec,None);
		}
		_ => return IcCommand::from_formated_vec(self.cmd.clone(),None),
		}
	}

	fn string_wrap(&self,s: String) -> String {
		if s.contains(char::is_whitespace) {"((".to_owned()+&s+"))"} else {s}
	}
}
impl Display for IcInputCommand<'_> {
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

///The input struct for IntelCache
pub struct IcInput {input_str: String,fmt_str: Vec<String>, pwd: i32, pwdstr: String}
impl IcInput {
	///Create a new empty IcInput
	pub fn new() -> IcInput {
		let mut proto_ici = IcInput { input_str: "".to_string(), fmt_str: Vec::new(),pwd: 0,pwdstr: "ROOT".to_string() };
		proto_ici.fmt_str.push(String::new());
		proto_ici
	}

	pub fn check_exit(&self) -> bool {
		return if self.fmt_str.len() > 0 && self.fmt_str[0] == "exit" {true} else {false};
	}
	pub fn flush(&mut self) {
		self.input_str = String::new();
		self.fmt_str = Vec::new();
	}
	pub fn prompt(&mut self) -> IcInputCommand {
		print!("{} > ",self.pwdstr);
		stdout().flush().unwrap();
		stdin().read_line(&mut self.input_str).expect("Error reading line");
		self.input_str = self.input_str.trim_end().to_string();
		IcInputCommand::from_input(self)
	}
	pub fn display(&self,p: IcPacket) {
		if p.header.as_ref().unwrap_or(&"None".to_string()) == "OK!" && p.body.as_ref().unwrap_or(&Vec::new()).len() > 0 {
			println!("{}",str::from_utf8(&p.body.unwrap()).unwrap());
		} else if p.header.as_ref().unwrap_or(&"None".to_string()) == "OK!" {
			println!("Nothing.");
		} else {println!("{}",p.header.as_ref().unwrap_or(&"None".to_string()));}
	}
	pub fn resp(&self,p: IcPacket) {
		if p.header.as_ref().unwrap_or(&"None".to_string()) == "OK!" {
			println!("Success!");
		} else {println!("Failed.")}
	}
	pub fn set_pwd(&mut self, pwdid: i32,client: &mut IcClient) -> bool {
		if pwdid < 0 {return false}
		else if pwdid == 0 {self.pwd = pwdid;self.pwdstr = "ROOT".to_string();return true;}
		let mut p = Vec::<String>::new();
		p.push("DIR".to_string());
		p.push("VALIDATE".to_string());
		p.push(pwdid.to_string());
		let icp = IcInputCommand::from_vec(self,p);
		
		let resp = client.send_cmd(&mut icp.to_ic_command());
		if resp.header.as_ref().unwrap() == "true" {
			self.pwdstr = str::from_utf8(&resp.body.unwrap()).unwrap().to_string();
			self.pwd = pwdid;
			return true;
		} else { return false; }
	}
}

