use crate::ic_types::ic_execute_mod::IcExecute;
use diesel::MysqlConnection;
use crate::ic_types::IcLoginDetails;
use crate::ic_types::IcError;

pub trait IcModule {
	fn get_name(&self) -> &str;
	fn get_version(&self) -> &str;
	fn get_command(&self,cmd: Vec<String>) -> Result<Box<dyn IcExecute<Connection = MysqlConnection,LoginDetails = Option<IcLoginDetails>>>,IcError>;

}
