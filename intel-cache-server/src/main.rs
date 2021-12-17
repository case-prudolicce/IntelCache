use intel_cache_lib::ichandler::ic_server::*;

static S:IcServer = IcServer{};

fn main() {
	S.listen();
}
