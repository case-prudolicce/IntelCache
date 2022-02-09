use crate::ic_types::IcPacket;

pub trait IcExecute {
	type Connection;
	
	fn exec(&mut self,con: &mut Self::Connection,cmd: Option<Vec<String>>,data: Option<Vec<u8>>,cached: bool) -> IcPacket;
	
	fn login_required(&mut self) -> bool;
}
