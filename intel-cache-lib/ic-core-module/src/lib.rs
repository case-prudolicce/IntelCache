pub mod core_login_command;
pub mod core_register_command;
pub use self::core_register_command::CoreRegister;
pub use self::core_login_command::CoreLogin;
use diesel::MysqlConnection;
use std::collections::HashMap;
use intel_cache_lib::ic_types::IcExecute;
use intel_cache_lib::ic_types::IcLoginDetails;
use intel_cache_lib::ic_types::IcModule;
use intel_cache_lib::ic_types::IcError;

pub struct IcCoreModule {name: String,version: String,e: HashMap<String,fn()->Box<dyn IcExecute<Connection = MysqlConnection,LoginDetails = Option<IcLoginDetails>>>>}
impl IcCoreModule {
	pub fn new() -> IcCoreModule {
		let mut ret = IcCoreModule { name: "CORE".to_string(), version: "1.0.0".to_string(), e: HashMap::new() };
		ret.load();
		ret
	}
	
	pub fn load(&mut self) {
		self.e.insert(
			"LOGIN".to_string(),
			CoreLogin::to_exe
		);
		self.e.insert(
			"REGISTER".to_string(),
			CoreRegister::to_exe
		);
	}
}
impl IcModule for IcCoreModule {
	fn get_name(&self) -> &str {
		&self.name
	}
	fn get_version(&self) -> &str {
		&self.version
	}
	fn get_command(&self,cmd: Vec<String>) -> Result<Box<dyn IcExecute<Connection = MysqlConnection,LoginDetails = Option<IcLoginDetails>>>,IcError> {
		for (name,f) in &self.e {
			if cmd[0] == *name {
				return Ok(f())
			}
		}
		return Err(IcError("COMMAND NOT FOUND".to_string()));
	}
}
