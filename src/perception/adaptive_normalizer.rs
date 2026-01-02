#[derive(Clone, Debug)]
pub struct AdaptiveNormalizer {
    count: u64,
    mean: Vec<f64>,
    m2: Vec<f64>, // Somme des carrés des différences (pour la variance)
    initialized: bool,
}

impl AdaptiveNormalizer {
    pub fn new() -> Self {
        AdaptiveNormalizer {
            count: 0,
            mean: Vec::new(),
            m2: Vec::new(),
            initialized: false,
        }
    }

    /// Met à jour les statistiques avec un nouveau vecteur brut
    pub fn update(&mut self, values: &[f64]) {
        // Initialisation paresseuse (Lazy) basée sur la dimension du premier vecteur
        if !self.initialized {
            self.mean = vec![0.0; values.len()];
            self.m2 = vec![0.0; values.len()];
            self.initialized = true;
        }

        assert_eq!(values.len(), self.mean.len(), "Dimension mismatch in stream");

        self.count += 1;

        // Algorithme de Welford pour chaque dimension
        for (i, x) in values.iter().enumerate() {
            let delta = x - self.mean[i];
            self.mean[i] += delta / self.count as f64;
            let delta2 = x - self.mean[i];
            self.m2[i] += delta * delta2;
        }
    }

    /// Transforme le vecteur brut en Z-Score : (x - mean) / std_dev
    pub fn normalize(&self, values: &[f64]) -> Vec<f64> {
        if self.count < 2 {
            // Pas assez de données pour la variance, on retourne centré ou brut
            // Ici on retourne brut pour ne pas casser le début du signal
            return values.to_vec(); 
        }

        let mut normalized = Vec::with_capacity(values.len());

        for (i, x) in values.iter().enumerate() {
            // Variance = M2 / (count - 1)
            let variance = self.m2[i] / (self.count - 1) as f64;
            let std_dev = variance.sqrt();

            if std_dev > 1e-9 {
                normalized.push((x - self.mean[i]) / std_dev);
            } else {
                // Si la variance est nulle (signal constant), on renvoie 0.0
                normalized.push(0.0);
            }
        }
        normalized
    }
}