use std::sync::Mutex;

use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use env_logger::Env;
use middleware::Authenticated;
use paperclip::actix::{
    web::{delete, post, resource},
    OpenApiExt,
};

use crate::util::data::{PersistentStorage, TemporaryStorage};

mod constants;
mod endpoints;
mod errors;
mod middleware;
mod structs;
mod traits;
mod util;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let persistent_db = Data::new(Mutex::new(PersistentStorage::new()));
    let temporary_db = Data::new(Mutex::new(TemporaryStorage::new()));

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .wrap_api()
            .wrap(Logger::default())
            .app_data(persistent_db.clone())
            .app_data(temporary_db.clone())
            .service(resource("/register").route(post().to(endpoints::register)))
            .service(
                resource("/me")
                    .wrap(Authenticated::new(temporary_db.clone()))
                    .route(delete().to(endpoints::delete_account)),
            )
            // OpenAPI spec:
            .with_json_spec_at("/spec/v2")
            .with_json_spec_v3_at("/spec/v3")
            .build()
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
