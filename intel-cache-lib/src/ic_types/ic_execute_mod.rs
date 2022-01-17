use crate::ic_types::IcPacket;

pub trait IcExecute {
	type Connection;
	type LoginDetails;
	
	fn exec(&mut self,con: Option<&mut Self::Connection>,login: &mut Self::LoginDetails) -> IcPacket;
	
	fn login_required(&mut self) -> bool;
}
