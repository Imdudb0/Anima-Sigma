use crate::perception::universal_vector::UniversalVector;

/*pub struct PrototypicalNeuralUnit {
    pub id: String,
    pub prototype_weight: UniversalVector,
    pub activation_threshold: f64,
    pub current_activation_energy: f64,
    pub link: Vec<String>,
    pub symbolic_label: String,
}*/

pub struct PrototypicalNeuralUnit {
    // Identifiant unique pour le PNU
    int id;
    
    // Vecteur de Poids (W) : "Chaque PNU possède en mémoire une 'Signature Prototype'. C'est son poids synaptique."
    float* weight_vector;
    size_t weight_dim;
    
    // Seuil d'Activation (θ) : "Seuil d'Activation (θ) : La tolérance du neurone (est-il strict ou laxiste ?)."
    float activation_threshold;
    
    // Valeur d'Activation (a_i) : "L'Activation du PNU (a_i) : Le PNU calcule sa résonance avec la signature."
    float activation_value;
    
    // Valeur de Vérité : "Ce 0.85 devient littéralement la Valeur de Vérité de la proposition atomique dans la logique de Lukasiewicz."
    float truth_value;
    
    // Étiquette Symbolique : "Chaque PNU possède une Étiquette Symbolique (un pointeur vers un concept logique, ex: Predicate:Falling)."
    const char* symbolic_label;
    
    // Liens Latéraux (L_ij) : "Liens Latéraux (L_ij) : Connexions vers les autres PNU (excitateurs ou inhibiteurs)."
    struct {
        int target_pnu_id;
        float weight; // positif=excitateur, négatif=inhibiteur
    }* lateral_links;
    size_t num_lateral_links;
    
    // Taux d'Apprentissage : "Quand une PNU gagne (reconnaît une signature), elle modifie légèrement son poids (W)"
    float learning_rate;
    
    // Corrélations Temporelles : "Si plusieurs PNU s'activent souvent ensemble (temporellement), une connexion physique se renforce"
    struct {
        int correlated_pnu_id;
        float correlation_strength;
    }* temporal_correlations;
    size_t num_correlations;
    
    // Signature Handle : "Le symbole FOL est un Pointeur vers l'adresse mémoire de la Signature."
    void* signature_handle;
};

/*impl PrototypicalNeuralUnit {
    pub fn new(prototype_weight: UniversalVector,  activation_threshold: f64) -> Self {
        Self {
            prototype_weight,
            activation_threshold,
            current_activation_energy: 0.0,
        }
    }

    pub fn hebbian_learning() {}
}*/