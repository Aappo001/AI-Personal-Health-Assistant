use core::panic;

use macros::response;

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
    eprintln!("{}", serde_json::to_string(&response!("User created", user)).unwrap());
    panic!()
}
