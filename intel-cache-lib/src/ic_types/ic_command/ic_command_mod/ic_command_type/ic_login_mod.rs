pub struct IcLogin {cmd: Vec<String>,}
impl IcLogin {
	pub fn new(args: Vec<String>) -> IcLogin {
		IcLogin { cmd: args }
	}
}
