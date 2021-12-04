extern crate diesel;
extern crate IntelCache;

use self::models::*;
use diesel::prelude::*;
use IntelCache::*;

fn main() {
	use self::schema::entry::dsl::*;
	
	let connection = establish_connection();
	show_entries(&connection,Some(true),Some(true));
}
