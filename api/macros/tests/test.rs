use core::panic;

use macros::{jres, response_gen};
use serde_json::json;

#[derive(serde::Serialize)]
pub struct User{
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[test]
fn it_works() {
    let user = User {
        name: "Sussy".to_string(),
        id: 727,
        email: "touching@grass.now".to_string()
    };
    eprintln!("{}", serde_json::to_string(&response_gen!("User created", user)).unwrap());
    panic!()
}
