use crate::perception::universal_vector::{UniversalVector, Signature, Gradient, Metadata};

pub struct UniversalTransducer;

impl UniversalTransducer {
    pub fn segment_and_process(raw: &[Vec<f64>], times: &Vec<f64>) -> Vec<UniversalVector> {
        if raw.len() < 2 { return vec![]; }
        let mut vectors = Vec::new();
        let mut start_idx = 0;
        let mut current_sign = 0.0; 

        for i in 1..raw.len() {
            let dx = raw[i][0] - raw[i-1][0];
            if dx.abs() < 1e-6 { continue; }
            let sign = dx.signum();

            if current_sign == 0.0 { current_sign = sign; }

            if sign != current_sign {
                let segment_raw = &raw[start_idx..i];
                let segment_times = &times[start_idx..i];
                if segment_raw.len() >= 3 {
                    vectors.push(Self::create_vector_from_slice(segment_raw, Some(segment_times.to_vec())));
                }
                start_idx = i - 1;
                current_sign = sign;
            }
        }

        if start_idx < raw.len() - 1 {
            let segment_raw = &raw[start_idx..];
            let segment_times = &times[start_idx..];
            if segment_raw.len() >= 3 {
                vectors.push(Self::create_vector_from_slice(segment_raw, Some(segment_times.to_vec())));
            }
        }

        vectors
    }

    fn create_vector_from_slice(raw: &[Vec<f64>], times: Option<Vec<f64>>) -> UniversalVector {
        assert!(!raw.is_empty(), "Raw data cannot be empty");
        let dim = raw[0].len();

        // 1. Calcul des incréments multidimensionnels (Deltas)
        let deltas: Vec<(f64, Vec<f64>)> = match times {
            Some(t) => {
                assert_eq!(t.len(), raw.len());
                raw.windows(2).zip(t.windows(2))
                    .map(|(w_raw, w_time)| {
                        let dt = w_time[1] - w_time[0];
                        let dX = w_raw[1].iter().zip(w_raw[0].iter())
                                         .map(|(x1, x0)| x1 - x0)
                                         .collect();
                        (dt, dX)
                    })
                    .collect()
            },
            None => raw.windows(2).map(|w| {
                let dX = w[1].iter().zip(w[0].iter())
                             .map(|(x1, x0)| x1 - x0)
                             .collect();
                (1.0, dX)
            }).collect(),
        };

        // 2. Accumulation via l'identité de Chen
        // Fix: Use dim + 1 because we often augment with time, 
        // but `Signature::zero` expects the signature dimension.
        // Based on `from_segment`, the signature dimension is (dim + 1).
        let mut current_signature = Signature::zero(dim + 1);

        for (dt, dX) in deltas.iter() {
            let segment_signature = Signature::from_segment(*dt, dX);
            current_signature = current_signature.combine(&segment_signature);
        }

        let gradient = Gradient::update(&deltas);

        UniversalVector {
            signature: current_signature,
            gradient,
            metadata: Metadata::zero(),
        }
    }
}