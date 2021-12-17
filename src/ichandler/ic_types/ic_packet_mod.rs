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
		return [self.header.as_ref().unwrap_or(&"".to_string()).len().to_string().as_bytes(),&[10_u8],self.header.as_ref().unwrap_or(&"".to_string()).as_bytes(),&[10_u8],self.body.as_ref().unwrap_or(&Vec::new()).len().to_string().as_bytes(),&[10_u8],&self.body.as_ref().unwrap_or(&Vec::new())].concat().to_vec();
	}
}
