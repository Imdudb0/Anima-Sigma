pub mod perception;
pub mod meta_cognition;

pub use meta_cognition::reflex::{
    ReflexMetrics,
    ReflexConfig,
};

pub use cortex::prototypical_neural_unit::{
    PrototypicalNeuralUnit,
    TopologyConfig,
    SignatureHandle,
    PNUState,
};