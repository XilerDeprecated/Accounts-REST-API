use crate::util::Database;
use actix_web::web::Data;
use std::sync::Arc;

pub type FullDatabase = Data<Arc<Database>>;
