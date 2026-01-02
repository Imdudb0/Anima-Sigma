use crate::perception::universal_vector::UniversalVector;

use std::f32::consts::E;

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
    // Pas besoin de "effective_threshold" : calculé à la volée dans la méthode `update()`

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