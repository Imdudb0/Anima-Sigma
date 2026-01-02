use ArchT3::{
    PrototypicalNeuralUnit, ReflexMetrics, ReflexConfig, PNUState, SignatureHandle, TopologyConfig,
};

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};


fn main() {
    let start_time = Instant::now();

    let metrics = Arc::new(Mutex::new(ReflexMetrics {
        actions_count: 0,
        average_response_time_ms: 0.0,
        errors: Vec::new(),
    }));

    let config = Arc::new(Mutex::new(ReflexConfig {
        reaction_threshold: 0.3,
        pattern: "default".to_string(),
        cooldown_ms: 500,
    }));

    // === THREAD 1: LE RÉFLEXE (Système 1) ===
    let config_reflex = Arc::clone(&config);
    let metrics_reflex = Arc::clone(&metrics);

    let reflex_handle = thread::spawn(move || {
        let mut local_actions = 0;

        loop {
            let elapsed = start_time.elapsed().as_secs_f64();
            if elapsed >= 1.0 { break; }

            // Récupère config actuelle (lecture rapide)
            let current_config = {
                let cfg = config_reflex.lock().unwrap();
                cfg.clone()
            };

            // Exécute l'action réflexe
            let action_start = Instant::now();

            // Simule un traitement rapide et automatique
            let random_input: f64 = rand::random();
            if random_input < current_config.reaction_threshold {
                println!("⚡ [{:6.2}s] Réflexe: Action immédiate (pattern: {})", 
                    elapsed, current_config.pattern);
                local_actions += 1;
            }

            let response_time_ms = action_start.elapsed().as_secs_f64() * 1000.0;

            {
                let mut met = metrics_reflex.lock().unwrap();
                met.actions_count += 1;
                met.average_response_time_ms = 
                    (met.average_response_time_ms * (met.actions_count - 1) as f64 + response_time_ms) 
                    / met.actions_count as f64;
            }

            thread::sleep(Duration::from_millis(current_config.cooldown_ms));
        }

        println!("⚡ Le réflexe s'arrête");
    });

    // === THREAD 2: LE STRATÈGE (Système 2) ===
    let config_strategist = Arc::clone(&config);
    let metrics_strategist = Arc::clone(&metrics);

    let strategist_handle = thread::spawn(move || {
        let mut last_analysis = 0.0;

        loop {
            let elapsed = start_time.elapsed().as_secs_f64();
            if elapsed >= 30.0 { break; }

            // Analyse toutes les 5 secondes
            if elapsed - last_analysis >= 5.0 {
                last_analysis = elapsed;

                let current_metrics = {
                    let met = metrics_strategist.lock().unwrap();
                    met.clone()
                };

                println!("\n🤔 === ANALYSE STRATÉGIQUE à t={:.1}s ===", elapsed);
                println!("   Actions totales: {}", current_metrics.actions_count);
                println!("   Temps moyen: {:.2}ms", current_metrics.average_response_time_ms);
                println!("   Erreurs: {}", current_metrics.errors.len());

                // LOGIQUE DE REPROGRAMMATION
                let mut new_config = {
                    let cfg = config_strategist.lock().unwrap();
                    cfg.clone()
                };

                // Ajuste en fonction des performances
                if current_metrics.average_response_time_ms > 10.0 {
                    new_config.cooldown_ms = (new_config.cooldown_ms as f64 * 0.8) as u64;
                    println!("   → Optimisation: cooldown réduit à {}ms", new_config.cooldown_ms);
                } else {
                    new_config.cooldown_ms = (new_config.cooldown_ms as f64 * 1.1) as u64;
                    println!("   → Sécurité: cooldown augmenté à {}ms", new_config.cooldown_ms);
                }

                // Change de stratégie
                new_config.pattern = match (elapsed as u32 / 5) % 3 {
                    0 => "agressif".to_string(),
                    1 => "defensif".to_string(),
                    _ => "equilibre".to_string(),
                };
                new_config.reaction_threshold = 0.2 + (elapsed / 60.0);

                println!("   → Nouveau pattern: {} (threshold: {:.2})", 
                    new_config.pattern, new_config.reaction_threshold);
                println!("=======================================\n");

                // Applique la reprogrammation
                {
                    let mut cfg = config_strategist.lock().unwrap();
                    *cfg = new_config;
                }
            }

            thread::sleep(Duration::from_millis(100));
        }

        println!("🤔 Le stratège termine son analyse");
    });

    let _ = reflex_handle.join();
    let _ = strategist_handle.join();

    // Résultats finaux
    let final_metrics = metrics.lock().unwrap();
    println!("\n📊 RÉSULTATS FINAUX (30s)");
    println!("Actions exécutées: {}", final_metrics.actions_count);
    println!("Performance moyenne: {:.2}ms/action", final_metrics.average_response_time_ms);
}


/*#[cfg(test)]
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
}*/