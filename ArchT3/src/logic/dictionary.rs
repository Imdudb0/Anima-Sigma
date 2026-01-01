use crate::perception::universal_vector::UniversalVector;

use std::sync::{LazyLock, Mutex};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Dictionary {
    pub concepts: HashMap<String, Vec<UniversalVector>>
}

pub static CONCEPTS: LazyLock<Mutex<Dictionary>> = LazyLock::new(|| {
    Mutex::new(Dictionary {
        concepts: HashMap::new(),
    })
});

impl Dictionary {
    pub fn resonate(prototype_weight: UniversalVector) {
        
    }
}