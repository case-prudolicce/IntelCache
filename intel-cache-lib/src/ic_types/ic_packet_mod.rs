/// Parsed `IcCommand` ready to be sent to the connection.
#[derive(Clone)]
pub struct IcPacket { 
	/// Header of the packet. Used for the Command or as a response.
	pub header: Option<String>,
	/// Body of the packet. Used for the Command data or as response data.
	pub body: Option<Vec<u8>> 
}
impl IcPacket {
	/// Construct new packet from `h` for the header and `d` for the body
	pub fn new(h: Option<String>, d: Option<Vec<u8>>) -> IcPacket {
		IcPacket { header: h, body: d }
	}
	/// Construct new empty packet 
	pub fn new_empty() -> IcPacket {
		IcPacket { header: None, body: None }
	}

	pub fn new_denied() -> IcPacket {
		IcPacket { header: Some("DENIED".to_owned()), body: None }
	}
	/// Return a vector representing the packet's header and body.
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
