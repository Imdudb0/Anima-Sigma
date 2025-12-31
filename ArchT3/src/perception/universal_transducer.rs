use crate::perception::universal_vector::{UniversalVector, Signature, Gradient, Metadata};

pub struct UniversalTransducer;

impl UniversalTransducer {
    /// Version classique (Intégrale Totale)
    pub fn process(raw: &[f64], times: Option<Vec<f64>>) -> UniversalVector {
        Self::create_vector_from_slice(raw, times)
    }

    /// NOUVEAU : Version Segmentée (Zero-Crossing)
    /// Découpe le signal quand la dynamique s'inverse (ex: Rebond)
    pub fn segment_and_process(raw: &[f64], times: &Vec<f64>) -> Vec<UniversalVector> {
        if raw.len() < 2 { return vec![]; }

        let mut vectors = Vec::new();
        let mut start_idx = 0;
        
        // On calcule les vitesses locales approximatives
        // Signe actuel (+1.0 ou -1.0)
        let mut current_sign = 0.0; 

        for i in 1..raw.len() {
            let dx = raw[i] - raw[i-1];
            // On ignore le bruit infinitésimal
            if dx.abs() < 1e-6 { continue; }

            let sign = dx.signum();

            // Initialisation du premier sens
            if current_sign == 0.0 {
                current_sign = sign;
            }

            // DETECTION DE RUPTURE (Changement de sens)
            if sign != current_sign {
                // 1. On cristallise le segment précédent (de start_idx à i)
                let segment_raw = &raw[start_idx..i];
                let segment_times = &times[start_idx..i];
                
                // On ne garde que les segments significatifs (> 3 points) pour éviter le bruit pur
                if segment_raw.len() >= 3 {
                    let vec = Self::create_vector_from_slice(segment_raw, Some(segment_times.to_vec()));
                    vectors.push(vec);
                }

                // 2. On reset pour le nouveau concept
                start_idx = i - 1; // On garde le point de pivot pour la continuité
                current_sign = sign;
            }
        }

        // Ne pas oublier le dernier morceau
        if start_idx < raw.len() - 1 {
            let segment_raw = &raw[start_idx..];
            let segment_times = &times[start_idx..];
            if segment_raw.len() >= 3 {
                vectors.push(Self::create_vector_from_slice(segment_raw, Some(segment_times.to_vec())));
            }
        }

        vectors
    }

    fn create_vector_from_slice(raw: &[f64], times: Option<Vec<f64>>) -> UniversalVector {
        assert!(!raw.is_empty(), "Raw data cannot be empty");

        // Calcul des incréments (Deltas)
        let deltas: Vec<(f64, f64)> = match times {
            Some(t) => {
                assert_eq!(t.len(), raw.len());
                raw.windows(2).zip(t.windows(2))
                    .map(|(w_raw, w_time)| (w_time[1] - w_time[0], w_raw[1] - w_raw[0]))
                    .collect()
            },
            None => raw.windows(2).map(|w| (1.0, w[1] - w[0])).collect(),
        };

        // Accumulation via l'identité de Chen itérative
        // On part de l'élément neutre (Identité)
        let mut current_signature = Signature::zero();

        for (dt, dx) in deltas.clone() {
            // 1. On calcule la signature géométrique locale du segment
            // (contient les termes 1/2 et 1/6 nécessaires à la convergence)
            let segment_signature = Signature::from_segment(dt, dx);

            // 2. On combine avec la signature accumulée précédente
            // Pour optimiser, on pourrait inliner le code de combine ici,
            // mais l'appel de fonction garantit la réussite du test de cohérence.
            current_signature = current_signature.combine(&segment_signature);
        }

        let gradient = Gradient::update(deltas);

        UniversalVector {
            signature: current_signature,
            gradient,
            metadata: Metadata::zero(),
        }
    }
}
