mod ic_execute_mod;
mod ic_command;
mod ic_packet_mod;
mod ic_connection_mod;

//pub use self::ic_execute_mod::IcExecute as IcExecute;
pub use self::ic_command::IcCommand as IcCommand;
pub use self::ic_packet_mod::IcPacket as IcPacket;
pub use self::ic_connection_mod::IcConnection as IcConnection;

/// Basic error for IntelCache
#[derive(Debug)]
pub struct IcError (
	/// Error message
	pub String
);

impl std::fmt::Display for IcError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl std::error::Error for IcError {}
