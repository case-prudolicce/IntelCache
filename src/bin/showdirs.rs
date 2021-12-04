extern crate diesel;
extern crate IntelCache;

use self::models::*;
use diesel::prelude::*;
use IntelCache::*;

fn main() {
	use self::schema::dir::dsl::*;
	
	let connection = establish_connection();
	show_dirs(&connection,Some(true));
}
