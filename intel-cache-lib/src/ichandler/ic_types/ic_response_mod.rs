#[derive(Clone)]
pub struct IcResponse { pub internal_val: (Option<i32>,Option<Vec<u8>>), }
impl IcResponse {
	pub fn from_str(string: String) -> IcResponse {
		IcResponse { internal_val: (Some(string.len() as i32),Some(string.as_bytes().to_vec())) }
	}
	pub fn data_get_response_from_str(string: String,length: i32) -> IcResponse {
		IcResponse { internal_val: (Some((length) * -1),Some(string.as_bytes().to_vec())) }
	}
	pub fn null_response() -> IcResponse {
		IcResponse { internal_val: (None,None) }
	}
	pub fn data_response(data: Vec<u8>) -> IcResponse {
		IcResponse { internal_val: (Some(data.len() as i32),Some([data.len().to_string().as_bytes(),&[10_u8],&data].concat())) }
	}
	pub fn data_get_response(data: Vec<u8>) -> IcResponse {
		IcResponse { internal_val: (Some((data.len() as i32) * -1),Some(data)) }
	}
	pub fn exit_response() -> IcResponse {
		IcResponse { internal_val: (Some(0),None) }
	}

	pub fn is_exit(&self) -> bool {
		if self.internal_val.0 != None && self.internal_val.0.unwrap() == 0 {true} else {false}
	}
	pub fn is_getting(&self) -> bool {
		if self.internal_val.0 != None && self.internal_val.0.unwrap() < 0 {true} else {false}
	}
	pub fn is_sending(&self) -> bool {
		if self.internal_val.0 != None && self.internal_val.0.unwrap() > 0 {true} else {false}
	}
	
	pub fn get_size(&self) -> i32 {
		return if self.internal_val.0.unwrap() < 0 {self.internal_val.0.unwrap()*-1} else {self.internal_val.0.unwrap()}
	}
}
