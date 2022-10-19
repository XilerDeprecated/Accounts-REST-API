use std::sync::Arc;

use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use env_logger::Env;
use middleware::AuthenticationService;
use paperclip::actix::{
    web::{delete, post, resource},
    OpenApiExt,
};
use types::FullDatabase;
use util::Database;

use crate::util::data::{PersistentStorage, TemporaryStorage};

mod constants;
mod endpoints;
mod errors;
mod middleware;
mod structs;
mod traits;
mod types;
mod util;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database = Database::new(TemporaryStorage::new(), PersistentStorage::new());
    let thread_db: FullDatabase = Data::new(Arc::new(database));

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .wrap_api()
            .wrap(Logger::default())
            .app_data(thread_db.clone())
            .service(resource("/register").route(post().to(endpoints::register)))
            .service(
                resource("/me")
                    .wrap(AuthenticationService::new(thread_db.clone()))
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
