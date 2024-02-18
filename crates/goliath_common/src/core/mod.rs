use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    Unsorted,
    Client,
    Vehicle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Client {
    pub id: String,
    pub secret_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: String,
    pub secret_key: String,
}
