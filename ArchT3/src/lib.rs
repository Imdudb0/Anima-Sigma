pub mod perception;
pub mod meta_cognition;

pub use meta_cognition::reflex::{
    ReflexMetrics,
    ReflexConfig,
};

pub use neural_swarm::prototypical_neural_unit::{
    PrototypicalNeuralUnit,
    TopologyConfig,
    SignatureHandle,
    PNUState,
};