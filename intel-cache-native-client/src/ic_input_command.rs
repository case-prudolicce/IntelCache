use intel_cache_lib::ic_types::{IcCommand};
use crate::ic_input::IcInput;
use std::fmt::Display;
use std::fmt;
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
			if self.cmd[0] == "new" {
				if self.cmd.len() >= 2 {
					fmt_vec.push(self.cmd[1].clone());
					fmt_vec.push("UNDER".to_string());
					fmt_vec.push(if self.cmd.len() == 3 {self.cmd[2].clone()} else {if self.ref_in.pwd != 0 {self.ref_in.pwd.to_string()} else {1.to_string()}});
				}
			}else if self.cmd[0] == "import" {
					fmt_vec.push(self.cmd[2].clone());
			}
			return IcCommand::from_formated_vec(fmt_vec,Some(self.databuff.clone()));
		},
		"get" => {
			fmt_vec.push("ENTRY".to_string());
			fmt_vec.push("GET".to_string());
			fmt_vec.push(self.string_wrap(self.cmd[1].clone()));
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
		"raw" => {
			return IcCommand::from_formated_vec(self.cmd[1..].to_vec().clone(),None);
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

