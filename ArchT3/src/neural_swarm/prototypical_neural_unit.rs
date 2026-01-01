use crate::perception::universal_vector::UniversalVector;

pub struct PrototypicalNeuralUnit {
    pub prototype_weight: UniversalVector,
    pub activation_threshold: f64,
    pub current_activation_energy: f64,
}