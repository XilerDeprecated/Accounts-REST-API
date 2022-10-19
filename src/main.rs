use std::sync::Mutex;

use actix_web::{web::Data, App, HttpServer};
use paperclip::actix::{
    web::{post, resource},
    OpenApiExt,
};

use crate::util::data::PersistentStorage;

mod endpoints;
mod errors;
mod structs;
mod traits;
mod util;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let persistent_db = Data::new(Mutex::new(PersistentStorage::new()));

    HttpServer::new(move || {
        App::new()
            .wrap_api()
            .app_data(persistent_db.clone())
            .service(resource("/register").route(post().to(endpoints::register)))
            // OpenAPI spec:
            .with_json_spec_at("/spec/v2")
            .with_json_spec_v3_at("/spec/v3")
            .build()
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
