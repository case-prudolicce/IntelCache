pub mod core_login_command;
pub mod core_register_command;
pub mod core_null_command;
pub mod core_fetch_command;
pub mod core_account_command;
pub use self::core_register_command::CoreRegister;
pub use self::core_login_command::CoreLogin;
pub use self::core_null_command::CoreNull;
pub use self::core_fetch_command::CoreFetch;
pub use self::core_account_command::CoreAccount;
use std::collections::HashMap;
use intel_cache_lib::ic_types::IcExecute;
use intel_cache_lib::ic_types::IcModule;
use intel_cache_lib::ic_types::IcError;
use intel_cache_lib::ic_types::IcConnection;

pub struct IcCoreModule {name: String,version: String,e: HashMap<String,fn()->Box<dyn IcExecute<Connection = IcConnection>>>}
impl IcModule for IcCoreModule {
	
	#[no_mangle]
	fn icm_load(&mut self) {
		self.e.insert(
			"LOGIN".to_string(),
			CoreLogin::cl_to_exe
		);
		self.e.insert(
			"REGISTER".to_string(),
			CoreRegister::cr_to_exe
		);
		self.e.insert(
			"NULL".to_string(),
			CoreNull::cn_to_exe
		);
		self.e.insert(
			"FETCH".to_string(),
			CoreFetch::cf_to_exe
		);
		self.e.insert(
			"ACCOUNT".to_string(),
			CoreAccount::ca_to_exe
		);
	}
	
	#[no_mangle]
	fn icm_get_name(&self) -> &str {
		&self.name
	}
	
	#[no_mangle]
	fn icm_get_version(&self) -> &str {
		&self.version
	}
	
	#[no_mangle]
	fn icm_get_command(&self,cmd: Vec<String>) -> Result<Box<dyn IcExecute<Connection = IcConnection>>,IcError> {
		for (name,f) in &self.e {
			if cmd[0] == *name {
				return Ok(f())
			}
		}
		return Err(IcError("COMMAND NOT FOUND".to_string()));
	}
}
#[no_mangle]
pub fn icm_new() -> *mut dyn IcModule {
	let mut ret = IcCoreModule { name: "CORE".to_string(), version: "1.0.0".to_string(), e: HashMap::new() };
	ret.icm_load();
	Box::into_raw(Box::new(ret))
}
