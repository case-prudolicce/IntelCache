extern crate diesel;
extern crate IntelCache;

use self::models::*;
use diesel::prelude::*;
use IntelCache::*;

fn main() {
	use self::schema::tag::dsl::*;
	
	let connection = establish_connection();
	let entryid = prompt_entry_target(&connection,Some("Entry to delete?".to_owned())).id;
	delete_entry(&connection,entryid);
	println!("Removed entry successfully.");
}
