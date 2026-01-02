use std::f64;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Clone, Serialize,  Deserialize)]
pub struct UniversalVector {
    pub signature: Signature,
    pub gradient: Gradient,
    pub metadata: Metadata,
}

#[derive(Debug, PartialEq, Clone,  Serialize,  Deserialize)]
pub struct Signature {
    pub dim: usize,
    pub level1: Vec<f64>,
    pub level2: Vec<Vec<f64>>,
    pub level3: Vec<Vec<Vec<f64>>>,
}

#[derive(Debug, PartialEq, Clone,  Serialize,  Deserialize)]
pub struct Gradient {
    data: Vec<(f64, Vec<f64>)>,
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
    pub fn resonance_directional(&self, other: &UniversalVector) -> f64 {
        let dot = self.signature.dot(&other.signature);
        let mag_self = self.signature.magnitude();
        let mag_other = other.signature.magnitude();

        if mag_self == 0.0 || mag_other == 0.0 {
            return 0.0;
        }
        dot / (mag_self * mag_other)
    }

    pub fn resonance_structural(&self, other: &UniversalVector, sigma: f64) -> f64 {
        let dist = self.signature.distance(&other.signature); 
        (- (dist * dist) / (2.0 * sigma * sigma)).exp()
    }

    pub fn resonance_full(&self, other: &UniversalVector, sensitivity: f64) -> f64 {
        let dir = self.resonance_directional(other);
        let struc = self.resonance_structural(other, sensitivity);
        let dir_clamped = dir.max(0.0);
        (dir_clamped * struc).sqrt()
    }

    pub fn normalize(&mut self) {
        self.signature.normalize();
    }

    pub fn to_normalized(&self) -> Self {
        let mut copy = self.clone();
        copy.normalize();
        copy
    }

    pub fn blend(&mut self, target: &UniversalVector, alpha: f64) {
        self.signature.blend(&target.signature, alpha);
    }

    pub fn zero() -> Self {
        UniversalVector {
            // Fix: Provided '0' as default dimension
            signature: Signature::zero(0),
            gradient: Gradient::zero(),
            metadata: Metadata::zero(),
        }
    }
}

impl Signature {
    pub fn from_segment(dt: f64, dX: &[f64]) -> Self {
        // Pour inclure le temps comme une dimension, on l'ajoute souvent au vecteur
        let mut d = vec![dt];
        d.extend_from_slice(dX);
        let actual_dim = d.len();

        let mut sig = Signature {
            dim: actual_dim, // Fix: Added missing field `dim`
            level1: d.clone(), // Fix: Changed from tuple (dt, dX[0]) to Vec<f64>
            level2: vec![vec![0.0; actual_dim]; actual_dim],
            level3: vec![vec![vec![0.0; actual_dim]; actual_dim]; actual_dim],
        };

        // Niveau 2
        for i in 0..actual_dim {
            for j in 0..actual_dim {
                sig.level2[i][j] = d[i] * d[j] / 2.0;
            }
        }

        // Niveau 3
        for i in 0..actual_dim {
            for j in 0..actual_dim {
                for k in 0..actual_dim {
                    sig.level3[i][j][k] = d[i] * d[j] * d[k] / 6.0;
                }
            }
        }
        sig
    }

    pub fn combine(&self, other: &Signature) -> Signature {
        assert_eq!(self.dim, other.dim, "Dimensions must match to combine signatures");
        let d = self.dim;
        let mut res = Signature::zero(d);

        // NIVEAU 1
        for i in 0..d {
            res.level1[i] = self.level1[i] + other.level1[i];
        }

        // NIVEAU 2
        for i in 0..d {
            for j in 0..d {
                res.level2[i][j] = self.level2[i][j] 
                                 + other.level2[i][j] 
                                 + (self.level1[i] * other.level1[j]);
            }
        }

        // NIVEAU 3
        for i in 0..d {
            for j in 0..d {
                for k in 0..d {
                    res.level3[i][j][k] = self.level3[i][j][k] 
                                        + other.level3[i][j][k]
                                        + (self.level1[i] * other.level2[j][k])
                                        + (self.level2[i][j] * other.level1[k]);
                }
            }
        }

        res
    }

    pub fn dot(&self, other: &Self) -> f64 {
        let mut sum = 0.0;
        let d = self.dim;
        for i in 0..d {
            sum += self.level1[i] * other.level1[i];
            for j in 0..d {
                sum += self.level2[i][j] * other.level2[i][j];
                for k in 0..d {
                    sum += self.level3[i][j][k] * other.level3[i][j][k];
                }
            }
        }
        sum
    }

    pub fn distance(&self, other: &Self) -> f64 {
        let mut sum_sq = 0.0;
        let d = self.dim;
        for i in 0..d {
            sum_sq += (self.level1[i] - other.level1[i]).powi(2);
            for j in 0..d {
                sum_sq += (self.level2[i][j] - other.level2[i][j]).powi(2);
                for k in 0..d {
                    sum_sq += (self.level3[i][j][k] - other.level3[i][j][k]).powi(2);
                }
            }
        }
        sum_sq.sqrt()
    }

    pub fn scale(&mut self, s: f64) {
        for i in 0..self.dim {
            self.level1[i] *= s;
            for j in 0..self.dim {
                self.level2[i][j] *= s;
                for k in 0..self.dim {
                    self.level3[i][j][k] *= s;
                }
            }
        }
    }

    pub fn blend(&mut self, target: &Signature, alpha: f64) {
        let a = alpha.clamp(0.0, 1.0);
        for i in 0..self.dim {
            self.level1[i] += (target.level1[i] - self.level1[i]) * a;
            for j in 0..self.dim {
                self.level2[i][j] += (target.level2[i][j] - self.level2[i][j]) * a;
                for k in 0..self.dim {
                    self.level3[i][j][k] += (target.level3[i][j][k] - self.level3[i][j][k]) * a;
                }
            }
        }
    }

    pub fn magnitude(&self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn normalize(&mut self) {
        let mag = self.magnitude();
        if mag > std::f64::EPSILON {
            let inv_mag = 1.0 / mag;
            self.scale(inv_mag);
        }
    }

    pub fn normalized_distance(&self, other: &Signature) -> f64 {
        let mut s1 = self.clone();
        let mut s2 = other.clone();
        s1.normalize();
        s2.normalize();
        s1.distance(&s2)
    }

    pub fn zero(dim: usize) -> Self {
        Signature {
            dim,
            level1: vec![0.0; dim],
            level2: vec![vec![0.0; dim]; dim],
            level3: vec![vec![vec![0.0; dim]; dim]; dim],
        }
    }
}

impl Gradient {
    pub fn update(deltas: &[(f64, Vec<f64>)]) -> Self {
        Gradient { data: deltas.to_vec() }
    }

    pub fn magnitude(&self) -> f64 {
        self.data.iter()
            .map(|(dt, dx_vec)| {
                let dx_sq_sum: f64 = dx_vec.iter().map(|x| x.powi(2)).sum();
                dt.powi(2) + dx_sq_sum
            })
            .sum::<f64>()
            .sqrt()
    }

    pub fn zero() -> Self {
        Gradient { data: Vec::new() }
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



/*Changez en utilisant la signature standard (aplatie) combinée à une projection aléatoire (Random Projection).
Remplacez le "trop d'allocations" par linearized signature.
Utilisez une signature linearisée sélective. Si input d est < 10 n'utilisez aucune compression sinon Utilisez la RP.*/