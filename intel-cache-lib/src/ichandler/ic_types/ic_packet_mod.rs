#[derive(Clone)]
pub struct IcPacket { pub header: Option<String>,pub body: Option<Vec<u8>> }
impl IcPacket {
	pub fn new(h: Option<String>, d: Option<Vec<u8>>) -> IcPacket {
		IcPacket { header: h, body: d }
	}
	pub fn new_empty() -> IcPacket {
		IcPacket { header: None, body: None }
	}
	
	pub fn pack(&self) -> Vec<u8> {
		let mut hasheader = false;
		let mut header: String = "".to_string();
		let mut body: Vec<u8> = Vec::new();
		let mut hasbody = false;
		match &self.header {
		Some(h) => {hasheader = true;header = h.to_string()},
		None => {},
		};

		match &self.body {
		Some(b) => {hasbody = true;body = b.to_vec()},
		None => {},
		};
		if hasheader && hasbody {
			return [header.len().to_string().as_bytes(),&[10_u8],header.as_bytes(),&[10_u8],body.len().to_string().as_bytes(),&[10_u8],&body].concat().to_vec();
		} else if hasheader {
			return [header.len().to_string().as_bytes(),&[10_u8],header.as_bytes(),&[10_u8],&[0_u8],&[10_u8]].concat().to_vec();
		} else {
			return [0_u8,10_u8,10_u8,0_u8,10_u8].to_vec();
		}
	}
}
