use actix_web::{App, HttpServer};
use paperclip::actix::{
    web::{post, resource},
    OpenApiExt,
};

mod endpoints;
mod structs;
mod traits;
mod util;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap_api()
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
