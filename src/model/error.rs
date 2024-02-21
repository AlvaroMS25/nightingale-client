use serde::Deserialize;

#[derive(Deserialize)]
pub struct ErrorResponse {
    pub message: String
}


