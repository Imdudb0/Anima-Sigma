Use std::f64;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Clone, Serialize,  Deserialize)]
pub struct UniversalVector {
    pub signature: Signature,
    pub gradient: Gradient,
    pub metadata: Metadata,
}

#[derive(Debug, PartialEq, Clone,  Serialize,  Deserialize)]
pub struct Signature {
    pub level1: (f64, f64),
    pub level2: [[f64; 2]; 2],
    pub level3: [[[f64; 2]; 2]; 2],
}

#[derive(Debug, PartialEq, Clone,  Serialize,  Deserialize)]
pub struct Gradient {
    data: Vec<(f64, f64)>,
}

#[derive(Debug, PartialEq, Clone,  Serialize,  Deserialize)]
pub enum Modality {
    Sensor,
    Vision,
    Audio,
    Memory,
}

#[derive(Debug, PartialEq, Clone,  Serialize,  Deserialize)]
pub struct Metadata {
    pub timestamp: f64,
    pub modality: Modality,
    pub reliability: f64,
}

impl UniversalVector {
    /// Résonance Directionnelle (Cosinus Similarité).
    /// Retourne une valeur entre -1.0 et 1.0.
    /// 1.0 signifie que les vecteurs codent exactement la même direction/forme (à une échelle près).
    /// 0.0 signifie qu'ils sont orthogonaux (aucune corrélation).
    pub fn resonance_directional(&self, other: &UniversalVector) -> f64 {
        let dot = self.signature.dot(&other.signature);
        let mag_self = self.signature.magnitude();
        let mag_other = other.signature.magnitude();

        if mag_self == 0.0 || mag_other == 0.0 {
            return 0.0; // Évite la division par zéro
        }

        // R = <A, B> / (||A|| * ||B||)
        dot / (mag_self * mag_other)
    }

    /// Résonance Structurelle (Noyau Gaussien / RBF).
    /// Retourne une valeur entre 0.0 et 1.0.
    /// Utilise la distance Euclidienne pour déterminer la proximité.
    /// Utile pour les mécanismes d'attention ou de clustering.
    ///
    /// * `sigma` : Sensibilité de la résonance (ex: 1.0). Plus sigma est petit, plus la résonance chute vite avec la distance.
    pub fn resonance_structural(&self, other: &UniversalVector, sigma: f64) -> f64 {
        // On utilise la méthode distance() qui existe déjà dans Signature
        let dist = self.signature.distance(&other.signature); 

        // R = exp(- distance^2 / (2 * sigma^2))
        (- (dist * dist) / (2.0 * sigma * sigma)).exp()
    }

    /// Résonance Hybride.
    /// Combine l'alignement directionnel et la proximité.
    /// Pondère la qualité de la forme et sa magnitude relative.
    pub fn resonance_full(&self, other: &UniversalVector, sensitivity: f64) -> f64 {
        let dir = self.resonance_directional(other);
        let struc = self.resonance_structural(other, sensitivity);

        // On ne garde que la résonance positive pour le mix
        let dir_clamped = dir.max(0.0);

        (dir_clamped * struc).sqrt() // Moyenne géométrique
    }

    /// Normalise le vecteur universel in-place.
    /// Après cet appel, `resonance_directional` avec ce vecteur sera plus stable,
    /// et la comparaison de formes de tailles différentes deviendra possible.
    pub fn normalize(&mut self) {
        self.signature.normalize();

        // Note : On ne normalise généralement pas le gradient ou les métadonnées
        // car ils portent des informations physiques absolues (vitesse, temps).
    }

    /// Crée une copie normalisée (style fonctionnel)
    pub fn to_normalized(&self) -> Self {
        let mut copy = self.clone();
        copy.normalize();
        copy
    }

    /// Met à jour le prototype entier (Signature + Gradient)
    pub fn blend(&mut self, target: &UniversalVector, alpha: f64) {
        self.signature.blend(&target.signature, alpha);
        // Note: On pourrait aussi blender le gradient, mais souvent 
        // on veut que le gradient reste une propriété de l'instance, pas du prototype.
        // Pour l'instant, on se concentre sur la FORME (Signature).
    }

    pub fn zero() -> Self {
        UniversalVector {
            signature: Signature::zero(),
            gradient: Gradient::zero(),
            metadata: Metadata::zero(),
        }
    }
}

impl Signature {
    /// Crée la signature exacte d'un segment linéaire (dx, dt)
    /// C'est crucial pour l'invariance au sous-échantillonnage.
    /// Un segment droit contient des termes d'ordre supérieur (1/2, 1/6) non nuls.
    pub fn from_segment(dt: f64, dx: f64) -> Self {
        let mut v = Self::zero();
        let vec = [dt, dx];

        // Niveau 1 : Le vecteur lui-même
        v.level1 = (dt, dx);

        // Niveau 2 : d ⊗ d / 2 (Intégration triangle sur segment droit)
        for i in 0..2 {
            for j in 0..2 {
                v.level2[i][j] = vec[i] * vec[j] * 0.5;
            }
        }

        // Niveau 3 : d ⊗ d ⊗ d / 6
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    v.level3[i][j][k] = vec[i] * vec[j] * vec[k] / 6.0;
                }
            }
        }
        v
    }

    /// Identité de Chen : S(ab) = S(a) ⊗ S(b)
    /// Cette fonction est l'unique source de vérité algébrique.
    pub fn combine(&self, other: &Signature) -> Signature {
        let mut res = Signature::zero();

        // --- NIVEAU 1 ---
        // S(ab)^1 = S(a)^1 + S(b)^1
        res.level1.0 = self.level1.0 + other.level1.0;
        res.level1.1 = self.level1.1 + other.level1.1;

        let sa1 = [self.level1.0, self.level1.1];
        let sb1 = [other.level1.0, other.level1.1];

        // --- NIVEAU 2 ---
        // S(ab)^2 = S(a)^2 + S(b)^2 + (S(a)^1 ⊗ S(b)^1)
        for i in 0..2 {
            for j in 0..2 {
                res.level2[i][j] = self.level2[i][j] 
                                 + other.level2[i][j] 
                                 + (sa1[i] * sb1[j]);
            }
        }

        // --- NIVEAU 3 ---
        // S(ab)^3 = S(a)^3 + S(b)^3 
        //         + (S(a)^1 ⊗ S(b)^2) 
        //         + (S(a)^2 ⊗ S(b)^1)
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    res.level3[i][j][k] = self.level3[i][j][k] 
                                        + other.level3[i][j][k]
                                        + (sa1[i] * other.level2[j][k])
                                        + (self.level2[i][j] * sb1[k]);
                }
            }
        }

        res
    }

    /// Calcule la distance L2 (Euclidienne) entre deux signatures.
    /// Aplatit les niveaux 1, 2 et 3 en un vecteur unique.
    pub fn distance(&self, other: &Signature) -> f64 {
        let mut sum_sq = 0.0;

        // Level 1
        sum_sq += (self.level1.0 - other.level1.0).powi(2);
        sum_sq += (self.level1.1 - other.level1.1).powi(2);

        // Level 2
        for i in 0..2 {
            for j in 0..2 {
                sum_sq += (self.level2[i][j] - other.level2[i][j]).powi(2);
            }
        }

        // Level 3
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    sum_sq += (self.level3[i][j][k] - other.level3[i][j][k]).powi(2);
                }
            }
        }

        sum_sq.sqrt()
    }

    /// Calcule le produit scalaire (Dot Product) entre deux signatures.
    /// <A, B> = Σ A_i * B_i
    pub fn dot(&self, other: &Self) -> f64 {
        let mut sum = 0.0;

        // Niveau 1
        sum += self.level1.0 * other.level1.0;
        sum += self.level1.1 * other.level1.1;

        // Niveau 2
        for i in 0..2 {
            for j in 0..2 {
                sum += self.level2[i][j] * other.level2[i][j];
            }
        }

        // Niveau 3
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    sum += self.level3[i][j][k] * other.level3[i][j][k];
                }
            }
        }

        sum
    }

    /// Calcule la magnitude (Norme L2) de la signature.
    /// ||A|| = sqrt(<A, A>)
    pub fn magnitude(&self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn resonance_directional(&self, other: &Signature) -> f64 {
        let dot = self.dot(&other);
        let mag_self = self.magnitude();
        let mag_other = other.magnitude();

        if mag_self == 0.0 || mag_other == 0.0 {
            return 0.0; // Évite la division par zéro
        }

        // R = <A, B> / (||A|| * ||B||)
        dot / (mag_self * mag_other)
    }

    /// Multiplie tous les termes de la signature par un scalaire `s`.
    /// Utilisé pour la normalisation ou pour simuler un changement d'échelle.
    pub fn scale(&mut self, s: f64) {
        // Niveau 1
        self.level1.0 *= s;
        self.level1.1 *= s;

        // Niveau 2
        for i in 0..2 {
            for j in 0..2 {
                self.level2[i][j] *= s;
            }
        }

        // Niveau 3
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    self.level3[i][j][k] *= s;
                }
            }
        }
    }

    /// Normalise la signature pour que sa magnitude (L2) soit égale à 1.0.
    /// Transforme le vecteur en "vecteur directionnel" pur (forme pure),
    /// en éliminant l'information d'amplitude absolue.
    pub fn normalize(&mut self) {
        let mag = self.magnitude();

        // Protection contre la division par zéro pour les vecteurs nuls
        if mag > std::f64::EPSILON {
            let inv_mag = 1.0 / mag;
            self.scale(inv_mag);
        }
    }

    /// Mélange la signature actuelle avec une cible (target) selon un facteur alpha.
    /// alpha = 0.0 -> On garde l'état actuel.
    /// alpha = 1.0 -> On devient la cible.
    /// Formule : S_new = S_old + alpha * (S_target - S_old)
    pub fn blend(&mut self, target: &Signature, alpha: f64) {
        let clamped_alpha = alpha.clamp(0.0, 1.0);

        // Level 1
        self.level1.0 += (target.level1.0 - self.level1.0) * clamped_alpha;
        self.level1.1 += (target.level1.1 - self.level1.1) * clamped_alpha;

        // Level 2
        for i in 0..2 {
            for j in 0..2 {
                self.level2[i][j] += (target.level2[i][j] - self.level2[i][j]) * clamped_alpha;
            }
        }

        // Level 3
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    self.level3[i][j][k] += (target.level3[i][j][k] - self.level3[i][j][k]) * clamped_alpha;
                }
            }
        }
    }

    /// Calcule la distance entre deux signatures après normalisation.
    /// Renvoie une valeur entre 0.0 (identique) et ~2.0 (opposé).
    /// C'est invariant à l'échelle du signal.
    pub fn normalized_distance(&self, other: &Signature) -> f64 {
        let mut s1 = self.clone();
        let mut s2 = other.clone();

        s1.normalize();
        s2.normalize();

        s1.distance(&s2)
    }

    pub fn zero() -> Self {
        Signature {
            level1: (0.0_f64, 0.0_f64),
            level2: [[0.0_f64; 2]; 2],
            level3: [[[0.0_f64; 2]; 2]; 2],
        }
    }
}

impl Gradient {
    pub fn update(deltas: Vec<(f64, f64)>) -> Self {
        Gradient { data: deltas }
    }

    pub fn zero() -> Self {
        Gradient { data: Vec::new() }
    }

    /// Calcule la magnitude (Norme L2) globale du gradient.
    /// Correspond à la racine carrée de la somme des carrés de tous les incréments (dt, dx).
    /// ||G|| = sqrt( Σ (dt^2 + dx^2) )
    pub fn magnitude(&self) -> f64 {
        self.data.iter()
            .map(|(dt, dx)| dt.powi(2) + dx.powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

impl Metadata {
    pub fn zero() -> Self {
        Metadata {
            timestamp: 0.0,
            modality: Modality::Vision,
            reliability: 1.0,
        }
    }
}