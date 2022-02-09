use std::fs::File;
use std::fs;

/// Parsed `IcCommand` ready to be sent to the connection.
#[derive(Clone)]
pub struct IcPacket { 
	/// Header of the packet. Used for the Command or as a response.
	pub header: Option<String>,
	/// Body of the packet. Used for the Command data or as response data.
	pub body: Option<Vec<u8>>,
	///set to true if body is cached.
	pub cached: bool
}
impl IcPacket {
	/// Construct new packet from `h` for the header and `d` for the body
	pub fn new(h: Option<String>, d: Option<Vec<u8>>) -> IcPacket {
		IcPacket { header: h, body: d, cached: false}
	}
	pub fn new_cached(h: Option<String>, d: Option<Vec<u8>>) -> IcPacket {
		IcPacket { header: h, body: d, cached: true}
	}
	/// Construct new empty packet 
	pub fn new_empty() -> IcPacket {
		IcPacket { header: None, body: None, cached: false}
	}

	pub fn new_denied() -> IcPacket {
		IcPacket { header: Some("DENIED".to_owned()), body: None, cached: false }
	}
	/// Return a vector representing the packet's header and body.
	pub fn pack(&self) -> Vec<u8> {
		let mut hasheader = false;
		let mut header: String = "".to_string();
		let mut body: Vec<u8> = Vec::new();
		let mut hasbody = false;
		let mut chunking = false;
		let mut chunked_body_len = 0;
		match &self.header {
			Some(h) => {hasheader = true;header = h.to_string()},
			None => {},
		};

		if ! &self.cached {
			match &self.body {
				Some(b) => {hasbody = true;body = b.to_vec()},
				None => {},
			};
		} else {
			let b = match std::str::from_utf8(&self.body.as_ref().unwrap()) {
				Ok(v) => v,
				Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
			};
			chunked_body_len = fs::metadata("./".to_owned()+b).unwrap().len();
			println!("PACKING BY CHUNKS FROM {} ({})",b,chunked_body_len);
			chunking = true;
			hasbody = true;
		}
		
		if hasheader && hasbody {
			if hasbody && ! chunking {
				return [header.len().to_string().as_bytes(),&[10_u8],header.as_bytes(),&[10_u8],body.len().to_string().as_bytes(),&[10_u8],&body].concat().to_vec();
			} else {
				return [header.len().to_string().as_bytes(),&[10_u8],header.as_bytes(),&[10_u8],chunked_body_len.to_string().as_bytes(),&[10_u8]].concat().to_vec();
			}
		} else if hasheader {
			return [header.len().to_string().as_bytes(),&[10_u8],header.as_bytes(),&[10_u8],&[0_u8],&[10_u8]].concat().to_vec();
		} else {
			return [0_u8,10_u8,10_u8,0_u8,10_u8].to_vec();
		}
	}
	
	pub fn parse_header(&self) -> Vec<String> {
		self.finalize_command(self.header.as_ref().unwrap_or(&String::new()).split_whitespace().collect::<Vec<&str>>())
	}

	fn finalize_command(&self,cmd: Vec<&str>) -> Vec<String> {
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
	
	pub fn from_parsed_header(input: Vec<String>,d: Option<Vec<u8>>) -> IcPacket {
		let i = IcPacket::unparse(input);
		IcPacket { header:Some(i),body: d, cached: false }
	}
	
	pub fn unparse(cmd: Vec<String>) -> String {
		let mut s = String::new();
		for t in &cmd {
			if t.contains(char::is_whitespace) {
				s.push_str(&("((".to_owned()+&t+")) "));
			}else {s.push_str(&(t.to_owned()+" "))}
		}
		s = s.trim_end().to_string();
		s
	}
}
