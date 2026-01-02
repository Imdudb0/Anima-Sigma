use crate::perception::adaptative_normalizer::AdaptiveNormalizer;
use crate::perception::universal_vector::UniversalVector;

use std::collections::VecDeque;

pub struct UniversalScanner {
    // Tampons pour le Transducer
    raw_buffer: Vec<Vec<f64>>,
    time_buffer: Vec<f64>,
    
    // Composants internes
    normalizer: AdaptiveNormalizer,
    batch_size: usize,
    
    // Option: pour geler l'apprentissage après une période de calibration ?
    learning_enabled: bool,
}

impl UniversalScanner {
    pub fn new(batch_size: usize) -> Self {
        UniversalScanner {
            raw_buffer: Vec::with_capacity(batch_size),
            time_buffer: Vec::with_capacity(batch_size),
            normalizer: AdaptiveNormalizer::new(),
            batch_size,
            learning_enabled: true,
        }
    }

    /// L'entrée principale : accepte n'importe quoi, apprend, normalise et stocke.
    pub fn ingest<T: UniversalSource>(&mut self, data: &T) {
        let raw_features = data.to_features();
        let timestamp = data.timestamp();

        // 1. Apprentissage (Welford Update)
        if self.learning_enabled {
            self.normalizer.update(&raw_features);
        }

        // 2. Normalisation immédiate
        // Note : Au tout début, cela retourne le brut tant que n < 2
        let processed_features = self.normalizer.normalize(&raw_features);

        // 3. Stockage
        self.raw_buffer.push(processed_features);
        self.time_buffer.push(timestamp);
    }

    /// Vérifie si on a assez de données pour lancer le Transducer
    pub fn is_ready(&self) -> bool {
        self.raw_buffer.len() >= self.batch_size
    }

    /// Génère les UniversalVectors et prépare le buffer suivant
    pub fn process_and_flush(&mut self) -> Vec<UniversalVector> {
        if !self.is_ready() { return vec![]; }

        // Appel au Transducer sur les données DÉJÀ normalisées
        let vectors = UniversalTransducer::segment_and_process(&self.raw_buffer, &self.time_buffer);

        // Gestion du chevauchement (Overlap)
        // On garde le dernier point pour assurer la continuité des dérivées (dX)
        if let (Some(last_raw), Some(last_time)) = (self.raw_buffer.last(), self.time_buffer.last()) {
            let last_r = last_raw.clone();
            let last_t = *last_time;
            
            self.raw_buffer.clear();
            self.time_buffer.clear();
            
            self.raw_buffer.push(last_r);
            self.time_buffer.push(last_t);
        } else {
            self.raw_buffer.clear();
            self.time_buffer.clear();
        }

        vectors
    }
    
    // Utile si on veut arrêter d'ajuster la moyenne/variance après un temps
    pub fn stop_learning(&mut self) {
        self.learning_enabled = false;
    }
}