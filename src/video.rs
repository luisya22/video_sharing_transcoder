use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Video {
    pub id: u64,
    pub name: String,
    pub path: String
}
