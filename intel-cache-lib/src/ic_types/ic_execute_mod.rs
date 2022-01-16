use crate::ic_types::IcPacket;

pub trait IcExecute {
	type Connection;
	
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> IcPacket;
	
	fn login_required(&mut self) -> bool;
}
