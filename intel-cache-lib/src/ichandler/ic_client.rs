use std::str;
use std::fmt;
use std::fs;
use std::process;
use std::net::TcpStream;
use std::fmt::Display;
use std::io::{stdout,stdin,ErrorKind,Error,Write};
use crate::ichandler::ic_types::{IcConnection,IcPacket,IcCommand};

#[derive(Debug)]
#[derive(PartialEq)]
pub enum IcClientMode {
	GET, 
	CAT, 
	SEND,
	EXIT,
	NONE,
}

///The input struct for IntelCache
pub struct IcInput { input_str: String,fmt_str: Vec<String>, pwd: i32, pwdstr: String}
impl IcInput {
	///Create a new empty IcInput
	pub fn new() -> IcInput {
		let mut proto_ici = IcInput { input_str: "".to_string(), fmt_str: Vec::new(),pwd: 0,pwdstr: "ROOT".to_string() };
		proto_ici.fmt_str.push(String::new());
		proto_ici
	}
	
	///Verify whether the internal input state is an exit state
	pub fn check_exit(&self) -> bool {
		return if self.fmt_str.len() > 0 && self.fmt_str[0] == "exit" {true} else {false};
	}
	///Flushes the IcInput
	pub fn flush(&mut self) {
		self.input_str = String::new();
		self.fmt_str = Vec::new();
	}
	///Prompts user for command. Will return an [`IcInputCommand`] (client side command).
	pub fn prompt(&mut self) -> IcInputCommand {
		print!("{} > ",self.pwdstr);
		stdout().flush().unwrap();
		stdin().read_line(&mut self.input_str).expect("Error reading line");
		self.input_str = self.input_str.trim_end().to_string();
		IcInputCommand::from_input(self)
	}
	
	///Sets the pwd (path working directory) for the IcInput.
	///This will get used by [`IcClient`] to append location info to
	///certain commands
	pub fn set_pwd(&mut self, pwdid: i32,client: &mut IcClient) -> bool {
		if pwdid < 0 {return false}
		else if pwdid == 0 {self.pwd = pwdid;self.pwdstr = "ROOT".to_string();return true;}
		let mut p = Vec::<String>::new();
		p.push("DIR".to_string());
		p.push("VALIDATE".to_string());
		p.push(pwdid.to_string());
		let icp = IcInputCommand::from_vec(self,p);
		
		
		
		client.con.send_packet(icp.to_ic_command().to_ic_packet()).unwrap();
		let resp = client.con.get_packet().unwrap();
		if resp.header.as_ref().unwrap() == "true" {
			self.pwdstr = str::from_utf8(&resp.body.unwrap()).unwrap().to_string();
			self.pwd = pwdid;
			return true;
		} else { return false; }
	}
}
/// The Client interface struct for IntelCache
pub struct IcClient { con: IcConnection,mode: IcClientMode }
impl IcClient {
	/// Connect to `ip` address
	///
	/// Note: the address is in ipv4 format. No ports.
	pub fn connect(ip: &str) -> Result<IcClient,Error> {
		let con = TcpStream::connect(ip.to_owned()+":64209");
		if let Ok(c) = con {
			return Ok(IcClient { con: IcConnection::new(c),mode: IcClientMode::CAT });
		} else {
			return Err(Error::new(ErrorKind::Other,"Failed to connect."));
		}
	}

	///`exec_cmd` will take a client side command for `c` ([`IcInputCommand`]),
	///translate it to a server side command and send it (if need be).
	///
	///Alternatively it can change internal values on `c`'s referring Input.
	pub fn exec_cmd(&mut self,c: &mut IcInputCommand) {
		self.update_mode(c);
		//Check connection
		if self.con.check_connection() {
			let mut sr: IcPacket = IcPacket::new_empty();
			if self.mode != IcClientMode::NONE {
				//println!("[DEBUG#IcClient.exec_cmd] SENDING IC_PACKET : {} ({:?})",c.to_ic_command().to_ic_packet().header.unwrap_or("None".to_string()),c.to_ic_command().to_ic_packet().body.unwrap().len());
				self.con.send_packet(c.to_ic_command().to_ic_packet()).unwrap(); 
				sr = self.con.get_packet().unwrap();
				//println!("[DEBUG#IcClient.exec_cmd] RECIEVING IC_PACKET : {} ({:?})",(&sr).header.as_ref().unwrap_or(&"None".to_string()),(&sr).body.as_ref().unwrap_or(&Vec::new()).len());
			}
			match self.mode {
			IcClientMode::CAT => {
				println!("{}",std::str::from_utf8(&sr.body.unwrap_or(Vec::new())).unwrap());
			},
			IcClientMode::GET => {
				fs::write(c.cmd[2].clone(),sr.body.unwrap()).unwrap();
			},
			IcClientMode::EXIT => {
				process::exit(1);
			},
			IcClientMode::NONE => {
				//println!("[DEBUG#IcClient.exec_cmd] Command putted client to NONE mode, not sending packets");
				if c.cmd[0] == "cd" {
					let result = if c.cmd.len() > 1 {
						c.ref_in.set_pwd(str::parse::<i32>(&c.cmd[1]).unwrap_or(-1),self)
					} else {c.ref_in.set_pwd(0,self)};
					if !result {println!("Not set!\n")} else {println!("Set!\n")};
				} 
			},
			_ => { },
			};
		} else {
			println!("Server closed.");
			process::exit(1);
		}
	}

	fn update_mode(&mut self,c: &IcInputCommand) {
		self.mode = match c.cmd[0].as_ref() {
		"new" | "set" | "mv" | "import" => IcClientMode::SEND,
		"exit" | "quit" => IcClientMode::EXIT,
		"cd" => IcClientMode::NONE,
		"get" => IcClientMode::GET,
		_ => IcClientMode::CAT, 
		}
	}
}

///A client side command. Used by [`IcClient`] to send [`IcCommand`]s to the [`crate::ichandler::ic_server::IcServer`]
///
///Each `IcInputCommand` has a reference to the [`IcInput`] that has generated it.
pub struct IcInputCommand<'a> { 
	///cmd is the internal parsed command.
	pub cmd: Vec<String>, 
	///databuff is the internal data associated with the command. This is usually file data.
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
