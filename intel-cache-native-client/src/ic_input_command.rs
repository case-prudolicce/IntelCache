use crate::ic_input::IcInput;
use std::fmt::Display;
use std::fmt;
use intel_cache_lib::ic_types::IcPacket;
use sha2::{Sha512, Digest};
pub struct IcInputCommand<'a> { 
	pub cmd: Vec<String>, 
	pub databuff: Option<Vec<u8>>,
	ref_in: &'a mut IcInput 
}
impl IcInputCommand<'_> {
	pub fn from_input(input: &mut IcInput) -> IcInputCommand {
		let mut con = false;
		let mut concatenated_str = String::new();
		let mut fcmd = Vec::new();
		if input.input_str.split_whitespace().collect::<Vec<&str>>().len() == 0 {
			return IcInputCommand { cmd:Vec::new(), databuff: None, ref_in: input}
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
		IcInputCommand { cmd:fcmd, databuff: None,ref_in: input }
	}
	pub fn from_vec<'a>(input: &'a mut IcInput,v: Vec<String>) -> IcInputCommand<'a> {
		let mut con = false;
		let mut concatenated_str = String::new();
		let mut fcmd = Vec::new();
		if v.len() == 0 {
			return IcInputCommand { cmd:Vec::new(), databuff: None, ref_in: input}
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
		IcInputCommand { cmd:fcmd, databuff: None,ref_in: input }
	}
	pub fn to_ic_packet(&self,cookie: &Option<String>) -> IcPacket {
		let mut fmt_vec:Vec<String> = Vec::new();
		match self.cmd[0].as_ref() {
		"new" | "import" => {
			fmt_vec.push("STORAGE".to_string());
			fmt_vec.push("ENTRY".to_string());
			fmt_vec.push("CREATE".to_string());
			if self.cmd[0] == "new" {
				if self.cmd.len() >= 2 {
					fmt_vec.push(self.cmd[1].clone());
					fmt_vec.push("PRIVATE".to_string());
					fmt_vec.push("UNDER".to_string());
					fmt_vec.push(if self.cmd.len() == 3 {self.cmd[2].clone()} else {if self.ref_in.pwd != 0 {self.ref_in.pwd.to_string()} else {"0".to_string()}});
				}
			}else if self.cmd[0] == "import" {
					fmt_vec.push(self.cmd[2].clone());
			}
			fmt_vec.push(cookie.as_ref().unwrap_or(&String::new()).to_string());
			return IcPacket::from_parsed_header(fmt_vec,self.databuff.clone());
		},
		"get" => {
			fmt_vec.push("STORAGE".to_string());
			fmt_vec.push("ENTRY".to_string());
			fmt_vec.push("GET".to_string());
			fmt_vec.push(self.string_wrap(self.cmd[1].clone()));
			fmt_vec.push(cookie.as_ref().unwrap_or(&String::new()).to_string());
			return IcPacket::from_parsed_header(fmt_vec,self.databuff.clone());
		},
		"ls" => {
			if self.cmd.len() >= 2 {
				if self.cmd[1].parse::<i32>().unwrap_or(-1) == -1 {
					match self.cmd[1].chars().nth(0).unwrap() {
					'f' => {
						fmt_vec.push("STORAGE".to_string());
						fmt_vec.push("ENTRY".to_string());
						fmt_vec.push("SHOW".to_string());
						fmt_vec.push("PRIVATE".to_string());
						if &self.cmd[1][1..] != "" {
							fmt_vec.push(self.cmd[1][1..].to_string());
						} else {
							fmt_vec.push("0".to_string());
						}
					},
					'd' => { //ls dir in the directory
						fmt_vec.push("STORAGE".to_string());
						fmt_vec.push("DIR".to_string());
						fmt_vec.push("SHOW".to_string());
						fmt_vec.push("PRIVATE".to_string());
						if &self.cmd[1][1..] != "" {
							fmt_vec.push(self.cmd[1][1..].to_string());
						} else {
							fmt_vec.push("0".to_string());
						}
					}
					'a' => { //ls all
						fmt_vec.push("STORAGE".to_string());
						fmt_vec.push("SHOW".to_string());
					}
					_ =>(),
					};
				} else { fmt_vec.push("STORAGE".to_string());fmt_vec.push("SHOW".to_string());fmt_vec.push(self.cmd[1].clone()) } //ls in the dir
			} else { //ls in the dir
				fmt_vec.push("STORAGE".to_string());
				fmt_vec.push("SHOW".to_string());
				fmt_vec.push(self.ref_in.pwd.to_string());
			}
			fmt_vec.push(cookie.as_ref().unwrap_or(&String::new()).to_string());
			return IcPacket::from_parsed_header(fmt_vec,self.databuff.clone());
		},
		"rm" => {
			fmt_vec.push("STORAGE".to_string());
			fmt_vec.push("ENTRY".to_string());
			fmt_vec.push("DELETE".to_string());
			fmt_vec.push(self.cmd[1].clone());
			fmt_vec.push(cookie.as_ref().unwrap_or(&String::new()).to_string());
			return IcPacket::from_parsed_header(fmt_vec,self.databuff.clone());
		},
		"set" => {
			if self.cmd.len() > 1 {
				fmt_vec.push("STORAGE".to_string());
				fmt_vec.push("ENTRY".to_string());
				fmt_vec.push("SET".to_string());
				fmt_vec.push(self.cmd[1].clone());
				fmt_vec.push(cookie.as_ref().unwrap_or(&String::new()).to_string());
				return IcPacket::from_parsed_header(fmt_vec,self.databuff.clone());
			}
			return IcPacket::from_parsed_header(fmt_vec,self.databuff.clone());
		},
		"mv" => {
			if self.cmd[1].chars().last().unwrap() == '/' {
				fmt_vec.push("STORAGE".to_string());
				fmt_vec.push("DIR".to_string());
				fmt_vec.push("SET".to_string());
				fmt_vec.push(self.cmd[1][..self.cmd[1].len() - 1].to_string());
				fmt_vec.push(self.cmd[2].clone());
			} else {
				fmt_vec.push("STORAGE".to_string());
				fmt_vec.push("ENTRY".to_string());
				fmt_vec.push("SET".to_string());
				fmt_vec.push(self.cmd[1].clone());
				fmt_vec.push(self.cmd[2].clone());
			}
			fmt_vec.push(cookie.as_ref().unwrap_or(&String::new()).to_string());
			return IcPacket::from_parsed_header(fmt_vec,self.databuff.clone());
		},
		"mkdir" => {
			fmt_vec.push("STORAGE".to_string());
			fmt_vec.push("DIR".to_string());
			fmt_vec.push("CREATE".to_string());
			fmt_vec.push(self.cmd[1].clone());
			fmt_vec.push("PRIVATE".to_string());
			fmt_vec.push("UNDER".to_string());
			fmt_vec.push(self.ref_in.pwd.to_string());
			fmt_vec.push(cookie.as_ref().unwrap_or(&String::new()).to_string());
			return IcPacket::from_parsed_header(fmt_vec,self.databuff.clone());
		},
		"rmdir" => {
			fmt_vec.push("STORAGE".to_string());
			fmt_vec.push("DIR".to_string());
			fmt_vec.push("DELETE".to_string());
			fmt_vec.push(self.cmd[1].clone());
			fmt_vec.push(cookie.as_ref().unwrap_or(&String::new()).to_string());
			return IcPacket::from_parsed_header(fmt_vec,self.databuff.clone());
		},
		"tag" => {
			if self.cmd[1].chars().last().unwrap() == '/' {
				fmt_vec.push("STORAGE".to_string());
				fmt_vec.push("TAG".to_string());
				fmt_vec.push("DIR".to_string());
				fmt_vec.push(self.cmd[1][..self.cmd[1].len() - 1].to_string());
				fmt_vec.push(self.cmd[2].clone());
			} else {
				fmt_vec.push("STORAGE".to_string());
				fmt_vec.push("TAG".to_string());
				fmt_vec.push("ENTRY".to_string());
				fmt_vec.push(self.cmd[1].clone());
				fmt_vec.push(self.cmd[2].clone());
			}
			fmt_vec.push(cookie.as_ref().unwrap_or(&String::new()).to_string());
			return IcPacket::from_parsed_header(fmt_vec,self.databuff.clone());
		},
		"untag" => {
			if self.cmd[1].chars().last().unwrap() == '/' {
				fmt_vec.push("STORAGE".to_string());
				fmt_vec.push("TAG".to_string());
				fmt_vec.push("UNDIR".to_string());
				fmt_vec.push(self.cmd[1][..self.cmd[1].len() - 1].to_string());
				fmt_vec.push(self.cmd[2].clone());
			} else {
				fmt_vec.push("STORAGE".to_string());
				fmt_vec.push("TAG".to_string());
				fmt_vec.push("UNENTRY".to_string());
				fmt_vec.push(self.cmd[1].clone());
				fmt_vec.push(self.cmd[2].clone());
			}
			fmt_vec.push(cookie.as_ref().unwrap_or(&String::new()).to_string());
			return IcPacket::from_parsed_header(fmt_vec,self.databuff.clone());
		},
		"tagrename" => {
			fmt_vec.push("STORAGE".to_string());
			fmt_vec.push("TAG".to_string());
			fmt_vec.push("RENAME".to_string());
			fmt_vec.push(self.cmd[1].clone());
			fmt_vec.push(self.cmd[2].clone());
			fmt_vec.push(cookie.as_ref().unwrap_or(&String::new()).to_string());
			return IcPacket::from_parsed_header(fmt_vec,self.databuff.clone());
		},
		"rename" => {
			if self.cmd[1].chars().last().unwrap() == '/' {
				fmt_vec.push("STORAGE".to_string());
				fmt_vec.push("DIR".to_string());
				fmt_vec.push("SET".to_string());
				fmt_vec.push(self.cmd[1][..self.cmd.len()-1].to_string());
				fmt_vec.push(self.cmd[2].clone());
				fmt_vec.push(cookie.as_ref().unwrap_or(&String::new()).to_string());
				return IcPacket::from_parsed_header(fmt_vec,self.databuff.clone());
			} else { 
				fmt_vec.push("STORAGE".to_string());
				fmt_vec.push("ENTRY".to_string());
				fmt_vec.push("SET".to_string());
				fmt_vec.push(self.cmd[1].clone());
				fmt_vec.push(self.cmd[2].clone());
				fmt_vec.push(cookie.as_ref().unwrap_or(&String::new()).to_string());
				return IcPacket::from_parsed_header(fmt_vec,self.databuff.clone());
			}
			
		}
		"showtags" => {
			fmt_vec.push("STORAGE".to_string());
			fmt_vec.push("TAG".to_string());
			fmt_vec.push("SHOW".to_string());
			fmt_vec.push(cookie.as_ref().unwrap_or(&String::new()).to_string());
			return IcPacket::from_parsed_header(fmt_vec,self.databuff.clone());
		},
		"mktag" => {
			fmt_vec.push("STORAGE".to_string());
			fmt_vec.push("TAG".to_string());
			fmt_vec.push("CREATE".to_string());
			fmt_vec.push(self.cmd[1].clone());
			fmt_vec.push(cookie.as_ref().unwrap_or(&String::new()).to_string());
			return IcPacket::from_parsed_header(fmt_vec,self.databuff.clone());
		},
		"rmtag" => {
			fmt_vec.push("STORAGE".to_string());
			fmt_vec.push("TAG".to_string());
			fmt_vec.push("DELETE".to_string());
			fmt_vec.push(self.cmd[1].clone());
			fmt_vec.push(cookie.as_ref().unwrap_or(&String::new()).to_string());
			return IcPacket::from_parsed_header(fmt_vec,self.databuff.clone());
		},
		"fetchusers" => {
			if self.cmd.len() > 1 {
				fmt_vec.push("CORE".to_string());
				fmt_vec.push("FETCH".to_string());
				fmt_vec.push("USER".to_string());
				fmt_vec.push(self.cmd[1].clone());
				return IcPacket::from_parsed_header(fmt_vec,None);
			}
			return IcPacket::from_parsed_header(fmt_vec,None);
		}
		"login" => {
			fmt_vec.push("CORE".to_string());
			fmt_vec.push("LOGIN".to_string());
			fmt_vec.push(self.cmd[1].clone());
			let p = self.cmd[2].clone();
			let mut hasher = Sha512::new();
			hasher.update(p);
			fmt_vec.push(format!("{:x}",hasher.finalize()));
			return IcPacket::from_parsed_header(fmt_vec,None);
		}
		"logout" => {
			fmt_vec.push("CORE".to_string());
			fmt_vec.push("ACCOUNT".to_string());
			fmt_vec.push("LOGOUT".to_string());
			fmt_vec.push(cookie.as_ref().unwrap_or(&String::new()).to_string());
			return IcPacket::from_parsed_header(fmt_vec,None);
		}
		"exit" => {
			fmt_vec.push("CORE".to_string());
			fmt_vec.push("EXIT".to_string());
			return IcPacket::from_parsed_header(fmt_vec,None);
		}
		"raw" => {
			return IcPacket::from_parsed_header(self.cmd[1..].to_vec().clone(),None);
		}
		_ => return IcPacket::from_parsed_header(self.cmd.clone(),None),
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

