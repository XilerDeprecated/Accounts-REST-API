use paperclip::actix::Apiv2Schema;
use serde::Serialize;

/// Represents a session
#[derive(Serialize, Apiv2Schema)]
pub struct Session {
    pub token: String,
    pub ttl: usize,
}
