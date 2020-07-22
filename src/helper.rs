use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Request{
    Set{key: String, value: String},
    Rm(String),
    Get(String)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SetResponse{
    Ok(()),
    Err(String)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RmResponse{
    Ok(()),
    Err(String)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GetResponse{
    Ok(Option<String>),
    Err(String)
}
