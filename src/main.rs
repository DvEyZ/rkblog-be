#[macro_use] 
extern crate rocket;

use dotenv;

mod routes;
mod db;
mod models;
mod errors;
mod middlewares;
use middlewares::{auth::SecretKeyWrapper};
use rocket::http::Method;
use rocket_cors::{CorsOptions, AllowedOrigins};
use routes::{post, user, auth};

#[launch]
async fn rocket() -> _ {
    dotenv::dotenv().ok();
    
    let rkt = rocket::build();

    let db_uri = std::env::var("RKBLOG_URI").unwrap();
    let db_name = std::env::var("RKBLOG_DATABASE").unwrap();
    let db = db::connect(&db_uri, &db_name).await.unwrap();
    let key = rocket::Config::from(rkt.figment()).secret_key;

    let cors = CorsOptions::default()
    .allowed_origins(AllowedOrigins::all())
    .allowed_methods(
        vec![Method::Get, Method::Post, Method::Patch, Method::Put, Method::Delete]
            .into_iter()
            .map(From::from)
            .collect(),
    )
    .allow_credentials(true)
    .to_cors().unwrap();

    rkt
    .attach(cors)
    .manage(SecretKeyWrapper {key})
    .manage(db.posts)
    .manage(db.users)
    .mount("/posts", routes![
        post::list, 
        post::get, 
        post::create, 
        post::update,
        post::delete
    ])
    .mount("/users", routes![
        user::list,
        user::get,
        user::create,
        user::update,
        user::delete
    ]).mount("/auth", routes![
        auth::get_token
    ]).register("/", catchers![
        errors::unauthorized,
        errors::forbidden
    ])
}