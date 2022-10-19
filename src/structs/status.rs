use std::fmt::Display;

use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};

/// Represents a status response.
#[derive(Serialize, Deserialize, Apiv2Schema, Debug)]
pub struct Status {
    pub message: String,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
