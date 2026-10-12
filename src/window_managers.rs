#![allow(non_local_definitions)]
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct WindowManager {
    pub name: String,
    pub exec: String,
}