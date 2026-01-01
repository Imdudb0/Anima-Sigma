use std::sync::{LazyLock, Mutex};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Dictionary {
    pub 
}

pub static CONCEPTS: LazyLock<Mutex<Dictionary>> = LazyLock::new(|| {
    Mutex::new(SwarmIndex {
        tribes: Vec::new(),
        orphans: Vec::new(),
    })
});