pub struct IcCoreModule {name: String,version: String,e: HashMap<String,fn()->IcExecute<Connection = MysqlConnection,LoginDetails = Option<IcLoginDetails>>>}
impl IcCoreModule {
	pub fn new() -> IcCoreModule {
		let ret = IcCoreModule { "CORE".to_string(), "1.0.0".to_string(), e: HashMap<String,fn()->IcExecute<Connection = MysqlConnection,LoginDetails = Option<IcLoginDetails>>::new()}
		ret.load()
		ret
	}
	
	pub fn load(&mut self) {
		self.e.insert(
			"LOGIN".to_string(),
			IcLogin::new
		};
		self.e.insert(
			"REGISTER".to_string(),
			IcRegister::new
		};
	}
}
impl IcModule for IcCoreModule {
	fn get_name(&self) -> String {
		self.name
	}
	fn get_version(&self) -> String {
		self.version
	}
	fn get_command(&self,cmd: Vec<String>) -> Result<Box<IcExecute<Connection = MysqlConnection,LoginDetails = Option<IcLoginDetails>>>,IcError> {
		for (name,f) in self.e {
			if cmd[0] == name {
				return Ok(Box::new(f()))
			}
		}
	}
}
