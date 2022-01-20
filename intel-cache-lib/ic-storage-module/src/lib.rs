//pub mod storage_show_command;
//pub mod storage_entry_command;
pub mod storage_dir_command;
//pub mod storage_tag_command;
//pub use self::storage_entry_command::StorageEntry;
pub use self::storage_dir_command::StorageDir;
//pub use self::storage_tag_command::StorageTag;
//pub use self::storage_show_command::StorageShow;

use diesel::MysqlConnection;
use std::collections::HashMap;
use intel_cache_lib::ic_types::IcExecute;
use intel_cache_lib::ic_types::IcLoginDetails;
use intel_cache_lib::ic_types::IcModule;
use intel_cache_lib::ic_types::IcError;
use intel_cache_lib::ic_types::IcConnection;

pub struct IcStorageModule {name: String,version: String,e: HashMap<String,fn()->Box<dyn IcExecute<Connection = IcConnection>>>}
impl IcModule for IcStorageModule {
	
	#[no_mangle]
	fn icm_load(&mut self) {
		//self.e.insert(
		//	"SHOW".to_string(),
		//	StorageShow::ss_to_exe
		//);
		//self.e.insert(
		//	"ENTRY".to_string(),
		//	StorageEntry::se_to_exe
		//);
		self.e.insert(
			"DIR".to_string(),
			StorageDir::sd_to_exe
		);
		//self.e.insert(
		//	"TAG".to_string(),
		//	StorageTag::st_to_exe
		//);
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
pub extern "C" fn icm_new() -> *mut dyn IcModule {
	let mut ret = IcStorageModule { name: "STORAGE".to_string(), version: "1.0.0".to_string(), e: HashMap::new() };
	ret.icm_load();
	Box::into_raw(Box::new(ret))
}
