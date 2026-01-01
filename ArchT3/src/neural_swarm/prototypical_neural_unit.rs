use crate::perception::universal_vector::UniversalVector;

pub struct PrototypicalNeuralUnit {
    pub id: String,
    pub prototype_weight: UniversalVector,
    pub activation_threshold: f64,
    pub current_activation_energy: f64,
    pub link: Vec<String>,
}

impl PrototypicalNeuralUnit {
    pub fn new(prototype_weight: UniversalVector,  activation_threshold: f64) -> Self {
        Self {
            prototype_weight,
            activation_threshold,
            current_activation_energy: 0.0,
        }
    }

    pub fn hebbian_learning() {}
}