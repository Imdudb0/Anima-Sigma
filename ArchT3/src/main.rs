use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// === STRUCTURES DE DONNÉES ===

#[derive(Clone)]
struct ReflexMetrics {
    actions_count: u32,
    average_response_time_ms: f64,
    errors: Vec<String>,
}

#[derive(Clone)]
struct ReflexConfig {
    reaction_threshold: f64,
    pattern: String,
    cooldown_ms: u64,
}

// === POINT D'ENTRÉE ===

fn main() {
    println!("🧠 Démarrage du cerveau artificiel...");
    println!("⚡ Système 1 (Réflexe) vs 🤔 Système 2 (Stratège)\n");

    let start_time = Instant::now();
    
    // Données partagées thread-safe
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
            if elapsed >= 30.0 { break; }

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

            // Met à jour les métriques
            {
                let mut met = metrics_reflex.lock().unwrap();
                met.actions_count += 1;
                met.average_response_time_ms = 
                    (met.average_response_time_ms * (met.actions_count - 1) as f64 + response_time_ms) 
                    / met.actions_count as f64;
            }

            thread::sleep(Duration::from_millis(current_config.cooldown_ms));
        }
        
        println!("⚡ Le réflexe s'arrête après 30s");
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

    // Attend la fin des deux threads
    let _ = reflex_handle.join();
    let _ = strategist_handle.join();

    // Résultats finaux
    let final_metrics = metrics.lock().unwrap();
    println!("\n📊 RÉSULTATS FINAUX (30s)");
    println!("Actions exécutées: {}", final_metrics.actions_count);
    println!("Performance moyenne: {:.2}ms/action", final_metrics.average_response_time_ms);
}