pub mod perception;
pub mod meta_cognition;

pub use meta_cognition::reflex::{
    ReflexMetrics,
    ReflexConfig,
};

pub use cognition::neural_swarm::{
    PrototypicalNeuralUnit,
    TopologyConfig,
    SignatureHandle,
};