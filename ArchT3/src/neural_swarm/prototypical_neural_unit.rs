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






// lateral_topology.rs
// Implementation of the "Essaim" Lateral Diffusion and Topological Inhibition.

use std::f32::consts::E;

// =============================================================================
// 1. Data Structures (Context Preserved)
// =============================================================================

#[derive(Clone, Debug)]
pub struct LateralLink {
    pub target_id: usize,
    pub weight: f32,          // Can be positive (Excitation) or negative (Inhibition)
    pub plasticity_rate: f32,
}

#[derive(Clone, Debug)]
pub struct TemporalCorrelation {
    pub pnu_id: usize,
    pub correlation_strength: f32,
    pub last_coactivation_time: f64,
}

/// Handle to raw signature (episodic memory)
#[derive(Clone, Debug)]
pub struct SignatureHandle {
    pub signature_segment: Vec<f32>,
    pub timestamp: f64,
    pub scene_context_id: u64,
}

#[derive(Clone, Debug)]
pub struct PNUState {
    pub activation: f32, // x_i
    pub derivative: f32, // dx_i/dt
}

#[derive(Clone, Debug)]
pub struct PrototypicalNeuralUnit {
    pub id: usize,
    pub symbolic_label: &'static str,

    pub state: PNUState,
    
    // W_i on unit sphere
    pub weight_vector: Box<[f32]>,   
    pub learning_rate_eta: f32,

    // Thresholds
    pub theta_base: f32,             
    pub theta_homeostatic: f32,      
    pub theta_semantic_fatigue: f32, 

    // Metabolic Budget
    pub activation_budget: f32,
    pub activation_consumption: f32,

    // Gain Control & Shunting
    pub auto_inhibition_a: f32,      // The "Leak" term A in Shunting Eq
    pub a_base: f32,
    pub gain_modulation_phi: f32,
    pub shunting_b: f32,             // Excitatory saturation bound
    pub shunting_c: f32,             // Inhibitory saturation bound
    pub decay_rate: f32,

    // Connectivity
    pub lateral_links: Vec<LateralLink>, 
    pub temporal_correlations: Vec<TemporalCorrelation>,

    pub signature_handle: SignatureHandle,

    // Logic Interface
    pub truth_value: f32,            
    pub injection_threshold: f32,    

    pub surprise_sensitivity: f32,
    pub vigilance_contribution: f32,

    pub last_spike_time: f64,
    pub last_surprise_time: f64,
    pub birth_timestamp: f64,
}

// =============================================================================
// 2. Topology Logic: Mexican Hat & Small World
// =============================================================================

/// Configuration for the Topological Generation
pub struct TopologyConfig {
    pub sigma_excitation: f32, // Width of excitatory peak
    pub sigma_inhibition: f32, // Width of inhibitory crown
    pub amp_excitation: f32,   // Height of excitation
    pub amp_inhibition: f32,   // Depth of inhibition
    pub connection_cutoff: f32,// Sparsity threshold (min absolute weight to keep link)
    pub max_neighbors: usize,  // Enforce O(sqrt(N)) sparsity
}

impl PrototypicalNeuralUnit {
    /// Calculates Euclidean distance between this PNU's prototype and another's.
    /// In Rough Paths space, this represents semantic distance.
    pub fn semantic_distance(&self, other: &PrototypicalNeuralUnit) -> f32 {
        self.weight_vector.iter()
            .zip(other.weight_vector.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    /// Checks the Gershgorin Circle Theorem condition for local stability.
    /// Condition: A_i > Sum(|z_ij|)
    /// If violated, it triggers Short-Term Plasticity (STP) to reduce lateral gain.
    pub fn enforce_gershgorin_stability(&mut self) {
        let lateral_influx_sum: f32 = self.lateral_links.iter()
            .map(|link| link.weight.abs())
            .sum();

        // Check dominance diagonal condition
        if lateral_influx_sum >= self.auto_inhibition_a {
            // Stability Violation Detected
            // We apply a scaling factor to satisfy A_i > Sum(|w|)
            // We leave a small epsilon margin for robust convergence
            let epsilon = 0.01;
            let scaling_factor = (self.auto_inhibition_a - epsilon) / lateral_influx_sum;

            // Apply STP (Gain Reduction)
            for link in &mut self.lateral_links {
                link.weight *= scaling_factor;
            }
            
            // Optionally: Log this event as "High Energy Stress"
        }
    }
}

/// Generates the "Mexican Hat" topology.
/// This connects semantically similar neurons (positive weights) and 
/// inhibits the semantic "crown" (negative weights).
/// 
pub fn wire_swarm_topology(swarm: &mut [PrototypicalNeuralUnit], config: &TopologyConfig) {
    let n = swarm.len();
    
    // Using indices to avoid borrowing conflicts
    for i in 0..n {
        let mut potential_links = Vec::with_capacity(n);

        for j in 0..n {
            if i == j { continue; }

            // 1. Calculate Semantic Distance (Distance in Signature Space)
            let dist = swarm[i].semantic_distance(&swarm[j]);

            // 2. Apply Mexican Hat Function (Difference of Gaussians)
            // w = A_e * exp(-d^2/s_e^2) - A_i * exp(-d^2/s_i^2)
            let excitation = config.amp_excitation * E.powf(-(dist.powi(2)) / (2.0 * config.sigma_excitation.powi(2)));
            let inhibition = config.amp_inhibition * E.powf(-(dist.powi(2)) / (2.0 * config.sigma_inhibition.powi(2)));
            
            let weight = excitation - inhibition;

            // 3. Sparsity Filter (Cutoff)
            if weight.abs() > config.connection_cutoff {
                potential_links.push(LateralLink {
                    target_id: swarm[j].id,
                    weight,
                    plasticity_rate: 0.01, // Base plasticity
                });
            }
        }

        // 4. Enforce Power Law / Small-World Sparsity
        // We only keep the strongest connections (both positive and negative)
        // to maintain O(sqrt(N)) complexity.
        potential_links.sort_by(|a, b| b.weight.abs().partial_cmp(&a.weight.abs()).unwrap());
        
        if potential_links.len() > config.max_neighbors {
            potential_links.truncate(config.max_neighbors);
        }

        swarm[i].lateral_links = potential_links;
        
        // 5. Pre-emptive Stability Check
        // Ensure initial wiring respects Gershgorin disks
        swarm[i].enforce_gershgorin_stability();
    }
}

// =============================================================================
// 3. Diffusion Dynamics (Runtime)
// =============================================================================

/// Calculates the lateral input term for the Shunting Equation.
/// Returns (Excitatory_Sum, Inhibitory_Sum)
/// Used in: dx/dt = -Ax + (B-x)E - (x+C)I
pub fn calculate_lateral_input(pnu: &PrototypicalNeuralUnit, swarm: &[PrototypicalNeuralUnit]) -> (f32, f32) {
    let mut exc_sum = 0.0;
    let mut inh_sum = 0.0;

    for link in &pnu.lateral_links {
        let neighbor = &swarm[link.target_id];
        
        // Assuming f(x) is sigmoid or ReLU. Here using simple max(0, x) for signal
        let signal = neighbor.state.activation.max(0.0); 

        if link.weight > 0.0 {
            // Excitation Voisine (Coopération)
            exc_sum += link.weight * signal;
        } else {
            // Inhibition Latérale (Compétition)
            // Note: link.weight is negative, so we take abs() for the Shunting I term
            inh_sum += link.weight.abs() * signal;
        }
    }
    
    (exc_sum, inh_sum)
}

// =============================================================================
// 4. Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a dummy PNU
    fn create_dummy_pnu(id: usize, coords: Vec<f32>) -> PrototypicalNeuralUnit {
        PrototypicalNeuralUnit {
            id,
            symbolic_label: "Test",
            state: PNUState { activation: 0.0, derivative: 0.0 },
            weight_vector: coords.into_boxed_slice(),
            learning_rate_eta: 0.01,
            theta_base: 0.5,
            theta_homeostatic: 0.0,
            theta_semantic_fatigue: 0.0,
            activation_budget: 100.0,
            activation_consumption: 0.0,
            auto_inhibition_a: 1.0, // Decay rate A
            a_base: 1.0,
            gain_modulation_phi: 0.1,
            shunting_b: 1.0,
            shunting_c: 0.2,
            decay_rate: 0.1,
            lateral_links: Vec::new(),
            temporal_correlations: Vec::new(),
            signature_handle: SignatureHandle { signature_segment: vec![], timestamp: 0.0, scene_context_id: 0 },
            truth_value: 0.0,
            injection_threshold: 0.8,
            surprise_sensitivity: 0.1,
            vigilance_contribution: 0.0,
            last_spike_time: 0.0,
            last_surprise_time: 0.0,
            birth_timestamp: 0.0,
        }
    }

    #[test]
    fn test_mexican_hat_topology() {
        // Create 3 PNUs: 
        // 0 and 1 are very close (should excite)
        // 0 and 2 are medium distance (should inhibit - The Crown)
        let mut swarm = vec![
            create_dummy_pnu(0, vec![1.0, 1.0]),
            create_dummy_pnu(1, vec![1.0, 1.1]), // Dist = 0.1
            create_dummy_pnu(2, vec![1.0, 3.0]), // Dist = 2.0
        ];

        let config = TopologyConfig {
            sigma_excitation: 0.5,
            sigma_inhibition: 1.5,
            amp_excitation: 2.0,
            amp_inhibition: 1.0,
            connection_cutoff: 0.01,
            max_neighbors: 10,
        };

        wire_swarm_topology(&mut swarm, &config);

        let pnu0 = &swarm[0];
        
        // Find link to PNU 1 (Close neighbor)
        let link_to_1 = pnu0.lateral_links.iter().find(|l| l.target_id == 1).unwrap();
        assert!(link_to_1.weight > 0.0, "Close neighbors should be excitatory");

        // Find link to PNU 2 (Crown neighbor)
        let link_to_2 = pnu0.lateral_links.iter().find(|l| l.target_id == 2).unwrap();
        assert!(link_to_2.weight < 0.0, "Medium distance neighbors should be inhibitory");

        println!("Link 0->1 Weight: {} (Expect +)", link_to_1.weight);
        println!("Link 0->2 Weight: {} (Expect -)", link_to_2.weight);
    }

    #[test]
    fn test_gershgorin_stability() {
        let mut pnu = create_dummy_pnu(0, vec![0.0]);
        // Set decay A = 1.0
        pnu.auto_inhibition_a = 1.0;

        // Manually add unstable links (Sum > 1.0)
        pnu.lateral_links.push(LateralLink { target_id: 1, weight: 0.8, plasticity_rate: 0.0 });
        pnu.lateral_links.push(LateralLink { target_id: 2, weight: -0.8, plasticity_rate: 0.0 }); // Abs = 0.8

        // Total Influx = 1.6 > 1.0 (Unstable!)
        
        pnu.enforce_gershgorin_stability();

        let new_sum: f32 = pnu.lateral_links.iter().map(|l| l.weight.abs()).sum();
        assert!(new_sum < pnu.auto_inhibition_a, "Gershgorin stability should scale down weights");
        println!("New lateral sum: {} < {}", new_sum, pnu.auto_inhibition_a);
    }
}
