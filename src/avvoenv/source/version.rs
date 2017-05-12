extern crate serde;
extern crate serde_json;

// #[macro_use]
// extern crate serde_derive;

use self::serde_json::Error;

#[derive(Deserialize)]
pub struct Version {
    pub version: i64,
    pub user: String,
    pub timestamp: String,
}
