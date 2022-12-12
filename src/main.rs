use std::vec;

use rocket::futures::TryStreamExt;
use rocket::serde::{self, json::Json, Deserialize, Serialize};
use rocket::{fs::FileServer, request::FromRequest, Request};
use rocket_db_pools::sqlx::{self, Row};
use rocket_db_pools::{Connection, Database};
use uuid::Uuid;

#[macro_use]
extern crate rocket;

#[derive(Database)]
#[database("wallet_management")]
struct WalletManagement(sqlx::PgPool);

#[get("/")]
async fn index() -> &'static str {
    println!("Hello world");
    "Hello, world!"
}

// #[get("/hello/<name>/<old>")]
// fn hello(name: &str, old: bool) -> String {
//     format!("Hello {} {}", name, old)
// }

#[derive(Deserialize, Serialize, FromForm)]
#[serde(crate = "rocket::serde")]
struct User {
    name: String,
}
#[post("/post", format = "application/json", data = "<user>")]
fn with_post(user: Json<User>) -> String {
    format!("Query: name: {}", user.name)
}

#[get("/user")]
fn get_user() -> Json<User> {
    Json(User {
        name: String::from("John Doe"),
    })
}

#[get("/user-demo?<user..>")]
fn get_user_demo(user: User) -> Json<User> {
    Json(user)
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct WalletAddress {
    id: String,
    address: String,
}

#[get("/addresses")]
async fn get_address(db: &WalletManagement) -> Json<Vec<WalletAddress>> {
    let mut list_address: Vec<WalletAddress> = Vec::new();
    let query = sqlx::query("SELECT * FROM wallets");
    let mut rows = query.fetch(&db.0);
    while let Some(row) = rows.try_next().await.unwrap() {
        let id_uuid: Uuid = row.get("id");
        let wallet_address = WalletAddress {
            id: id_uuid.to_string(),
            address: row.get("address"),
        };
        list_address.push(wallet_address);
    }

    Json(list_address)
}

#[get("/addresses/<address_id>")]
async fn get_address_by_id(db: &WalletManagement, address_id: &str) -> Json<Vec<WalletAddress>> {
    let mut list_address: Vec<WalletAddress> = Vec::new();
    println!("Query address_id: {}", address_id);
    let my_str_query = format!("Select * From wallets where address = '{}'", address_id);
    println!("Query: {}", my_str_query);
    let query = sqlx::query(my_str_query.as_str());
    let mut rows = query.fetch(&db.0);
    while let Some(row) = rows.try_next().await.unwrap() {
        let id: Uuid = row.get("id");
        let wallet_address = WalletAddress {
            id: id.to_string(),
            address: row.get("address"),
        };
        list_address.push(wallet_address);
    }

    Json(list_address)
}

#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("Sorry, '{}' is not a valid path.", req.uri())
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .attach(WalletManagement::init())
        .register("/", catchers![not_found])
        .mount(
            "/",
            routes![
                index,
                with_post,
                get_user,
                get_user_demo,
                get_address,
                get_address_by_id
            ],
        )
        .mount("/public", FileServer::from("public/"))
        .launch()
        .await?;

    Ok(())
}
