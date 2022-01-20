use intel_cache_lib::ic_types::IcPacket;
//use crate::ic_types::IcExecute;
use intel_cache_lib::ic_types::ic_execute_mod::IcExecute;
use intel_cache_lib::ic_types::IcConnection;

pub struct CoreNull {}
impl CoreNull {
	#[no_mangle]
	pub fn cn_new() -> CoreNull {
		CoreNull { }
	}
	
	#[no_mangle]
	pub fn cn_to_exe() -> Box<dyn IcExecute<Connection = IcConnection>> {
		Box::new(CoreNull::cn_new())
	}
}
impl IcExecute for CoreNull {
	type Connection = IcConnection;
	fn exec(&mut self,_con: &mut Self::Connection,_cmd: Option<Vec<String>>,_data: Option<Vec<u8>>) -> IcPacket {
		IcPacket::new_empty()
	}
	
	fn login_required(&mut self) -> bool {
		false
	}
}
