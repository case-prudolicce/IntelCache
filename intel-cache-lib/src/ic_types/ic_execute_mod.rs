use crate::ic_types::IcPacket;

pub trait IcExecute {
	type Connection;
	
	fn exec(&mut self,con: &mut Self::Connection,cmd: Option<Vec<String>>) -> IcPacket;
	
	fn login_required(&mut self) -> bool;
}
