use crate::perception::universal_vector::UniversalVector;

#[derive(Clone)]
pub struct LateralLink {
    pub target_id: usize,
    pub weight: f32,
    pub plasticity_rate: f32,
}

#[derive(Clone)]
pub struct TemporalCorrelation {
    pub pnu_id: usize,
    pub correlation_strength: f32,
    pub last_coactivation_time: f64,
}

/// Handle vers signature brute (mémoire épisodique)
pub struct SignatureHandle {
    pub signature_segment: Vec<f32>,
    pub timestamp: f64,
    pub scene_context_id: u64,
}

pub struct PNUState {
    pub activation: f32,
    pub derivative: f32,
}

pub struct PrototypicalNeuralUnit {
    pub id: usize,
    pub symbolic_label: &'static str,

    pub state: PNUState,

    // Poids prototype & apprentissage Oja
    pub weight_vector: Box<[f32]>,   // W_i sur sphère unité (pas de *float!)
    pub learning_rate_eta: f32,

    // Seuils multi-modulés (Rust garantit l'initialisation)
    pub theta_base: f32,             // Tolérance de base
    pub theta_homeostatic: f32,      // Δθ par homéostasie
    pub theta_semantic_fatigue: f32, // μ_i : fatigue par surprise
    // Pas besoin de "effective_threshold" : calculé à la volée dans la méthode `update()`

    // Budget métabolique (sécurisé contre la saturation)
    pub activation_budget: f32,
    pub activation_consumption: f32,

    // Contrôle de gain temporel (hystérèse)
    pub auto_inhibition_a: f32,
    pub a_base: f32,
    pub gain_modulation_phi: f32,

    // Paramètres Shunting (B, C, A)
    pub shunting_b: f32,
    pub shunting_c: f32,
    pub decay_rate: f32,

    // Connectivité latérale (Vec au lieu de tableau C)
    pub lateral_links: Vec<LateralLink>, // ~√N voisins (Small-World)

    // Corrélations temporelles
    pub temporal_correlations: Vec<TemporalCorrelation>,

    // Handle vers signature brute
    pub signature_handle: SignatureHandle,

    // Valeurs FOL
    pub truth_value: f32,            // Pour logique de Lukasiewicz
    pub injection_threshold: f32,    // Seuil de cristallisation

    // Plasticité spécialisée
    pub surprise_sensitivity: f32,
    pub vigilance_contribution: f32,

    // Timestamps
    pub last_spike_time: f64,
    pub last_surprise_time: f64,
    pub birth_timestamp: f64,
}