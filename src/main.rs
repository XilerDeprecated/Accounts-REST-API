use actix_web::{App, HttpServer};
use paperclip::actix::OpenApiExt;

mod endpoints;
mod util;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap_api()
            // OpenAPI spec:
            .with_json_spec_at("/spec/v2")
            .with_json_spec_v3_at("/spec/v3")
            .build()
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
