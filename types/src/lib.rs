use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    pub ts: i64,
    pub text: String,
    pub sender: String,
}
