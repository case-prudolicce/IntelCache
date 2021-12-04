extern crate diesel;
extern crate IntelCache;

use self::models::*;
use diesel::prelude::*;
use IntelCache::*;

fn main() {
	use self::schema::tag::dsl::*;
	
	let connection = establish_connection();
	delete_tag(&connection);
}
