use crate::ichandler::ic_types::IcPacket;

pub trait IcExecute {
	type Connection;
	
	fn exec(&mut self,con: Option<&mut Self::Connection>) -> IcPacket;
}
