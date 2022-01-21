use crate::ic_types::{ic_execute_mod::IcExecute,IcError,IcConnection};

pub trait IcModule {
	fn icm_load(&mut self);
	fn icm_get_name(&self) -> &str;
	fn icm_get_version(&self) -> &str;
	fn icm_get_command(&self,cmd: Vec<String>) -> Result<Box<dyn IcExecute<Connection = IcConnection>>,IcError>;

}
