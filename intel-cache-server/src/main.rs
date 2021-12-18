use intel_cache_lib::IcServer;

static S:IcServer = IcServer{};

fn main() {
	S.listen();
}
