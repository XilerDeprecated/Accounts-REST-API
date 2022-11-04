use std::sync::Arc;

// use actix_cors::Cors;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use env_logger::Env;
use middleware::AuthenticationService;
use paperclip::actix::{
    web::{delete, get, post, resource},
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
    let database = Database::new(
        PersistentStorage::new().await,
        TemporaryStorage::new().await,
    );
    let thread_db: FullDatabase = Data::new(Arc::new(database));

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(move || {
        // let cors = Cors::default()
        //     .allowed_origin("http://localhost:80")
        //     .allowed_methods(vec!["GET", "POST", "DELETE"])
        //     .max_age(3600);

        App::new()
            .wrap_api()
            // .wrap(cors)
            .wrap(Logger::default())
            .app_data(thread_db.clone())
            .service(resource("/register").route(post().to(endpoints::register)))
            .service(resource("/login").route(post().to(endpoints::add_login)))
            .service(
                resource("/me")
                    .wrap(AuthenticationService::new(thread_db.clone()))
                    .route(delete().to(endpoints::delete_account))
                    .route(get().to(endpoints::get_account)),
            )
            .service(
                resource("/logout")
                    .wrap(AuthenticationService::new(thread_db.clone()))
                    .route(delete().to(endpoints::logout)),
            )
            .service(
                resource("/verify")
                    .wrap(AuthenticationService::new(thread_db.clone()))
                    .route(get().to(endpoints::verify_user)),
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
