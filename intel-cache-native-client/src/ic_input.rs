use std::fs;
use std::str;
use intel_cache_lib::IcClient;
use intel_cache_lib::ic_types::IcPacket;
use crate::ic_input_command::IcInputCommand;
use std::io::stdin;
use std::io::stdout;
use std::io::Write;
pub struct IcInput {pub cookie: Option<String>,pub input_str: String,pub fmt_str: Vec<String>, pub pwd: i32, pub pwdstr: String}
impl IcInput {
	pub fn new() -> IcInput {
		let mut proto_ici = IcInput { cookie: None,input_str: "".to_string(), fmt_str: Vec::new(),pwd: 0,pwdstr: "ROOT".to_string() };
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
		if p.header.as_ref().unwrap_or(&"None".to_string()) != "None" && p.body.as_ref().unwrap_or(&Vec::new()).len() > 0 {
			println!("{}",str::from_utf8(&p.body.unwrap()).unwrap());
		} else if p.header.as_ref().unwrap_or(&"None".to_string()) == "OK!" {
			println!("Nothing.");
		} else {println!("Failed.");}
	}
	pub fn resp(&self,p: IcPacket) {
		if p.header.as_ref().unwrap_or(&"None".to_string()) == "OK!" {
			println!("Success!");
		} else {println!("Failed.")}
	}
	pub fn debug(&self,p: IcPacket) {
		println!("{} : {}",p.header.unwrap_or("None".to_string()),p.body.unwrap_or(Vec::new()).len());
	}
	pub fn write_to_file(p: IcPacket,name: String) {
		if p.header.as_ref().unwrap_or(&"None".to_string()) == "OK!" {
			if p.body.as_ref().unwrap_or(&Vec::new()).len() > 0 {
				let data = p.body.unwrap();
				let r = fs::write(name,data);
				match r {
				Ok(_e) => println!("Success!"),
				Err(err) => panic!("{}",err),
				}
			} else {
				println!("Response is empty.");
			}
		} else {
			println!("Failed.");
		}
	}
	pub fn set_pwd(&mut self, pwdid: i32,client: &mut IcClient,cookie: &Option<String>) -> bool {
		if pwdid < 0 {return false}
		else if pwdid == 0 {self.pwd = pwdid;self.pwdstr = "ROOT".to_string();return true;}
		let mut p = Vec::<String>::new();
		if let c = cookie.as_ref().unwrap(){
			p.push("STORAGE".to_string());
			p.push("DIR".to_string());
			p.push("VALIDATE".to_string());
			p.push(pwdid.to_string());
			p.push(c.to_string());
			let icp = IcInputCommand::from_vec(self,p);
			
			
			let resp = client.send_cmd(&mut icp.to_ic_packet(cookie));
			if resp.header.as_ref().unwrap() == "true" {
				self.pwdstr = str::from_utf8(&resp.body.unwrap()).unwrap().to_string();
				self.pwd = pwdid;
				return true;
			} else { return false; }
		} else {return false}
	}
}

