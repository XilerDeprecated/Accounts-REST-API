use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};

/// Represents a status response.
#[derive(Serialize, Deserialize, Apiv2Schema)]
pub struct Status {
    pub code: u32,
    pub message: String,
}
