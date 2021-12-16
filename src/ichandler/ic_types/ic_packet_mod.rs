#[derive(Clone)]
pub struct ic_packet { pub header: Option<String>,pub body: Option<Vec<u8>> }
impl ic_packet {
	pub fn new(h: Option<String>, d: Option<Vec<u8>>) -> ic_packet {
		ic_packet { header: h, body: d }
	}
	pub fn new_empty() -> ic_packet {
		ic_packet { header: None, body: None }
	}
	
	pub fn pack(&self) -> Vec<u8> {
		return [self.header.as_ref().unwrap_or(&"".to_string()).len().to_string().as_bytes(),&[10_u8],self.header.as_ref().unwrap_or(&"".to_string()).as_bytes(),&[10_u8],self.body.as_ref().unwrap_or(&Vec::new()).len().to_string().as_bytes(),&[10_u8],&self.body.as_ref().unwrap_or(&Vec::new())].concat().to_vec();
	}
}
