use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserCollection {
    pub name: String,
}
