use rapier3d::prelude::*;
use rand::Rng;

/// Structure représentant l'animation d'un bâton en équilibre sur un chariot
pub struct BalanceStickAnimation {
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    cart_handle: RigidBodyHandle,
    stick_handle: RigidBodyHandle,
    duration: f64,
    time_step: f64,
}

impl BalanceStickAnimation {
    /// Crée une nouvelle simulation avec une durée donnée
    pub fn new(duration: f64) -> Self {
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();
        
        // Création du sol
        let ground_body = RigidBodyBuilder::fixed()
            .translation(vector![0.0, -0.5, 0.0])
            .build();
        let ground_handle = rigid_body_set.insert(ground_body);
        let ground_collider = ColliderBuilder::cuboid(50.0, 0.5, 50.0).build();
        collider_set.insert_with_parent(ground_collider, ground_handle, &mut rigid_body_set);
        
        // Création du chariot (avec friction pour le sol)
        let cart_body = RigidBodyBuilder::dynamic()
            .translation(vector![0.0, 0.5, 0.0])
            .lock_rotations() // Le chariot ne peut pas tourner
            .build();
        let cart_handle = rigid_body_set.insert(cart_body);
        let cart_collider = ColliderBuilder::cuboid(1.0, 0.2, 0.5)
            .friction(0.3)
            .restitution(0.1)
            .build();
        collider_set.insert_with_parent(cart_collider, cart_handle, &mut rigid_body_set);
        
        // Création du bâton (longue tige verticale)
        let stick_length = 3.0;
        let stick_body = RigidBodyBuilder::dynamic()
            .translation(vector![0.0, 0.7 + stick_length / 2.0, 0.0])
            .build();
        let stick_handle = rigid_body_set.insert(stick_body);
        let stick_collider = ColliderBuilder::cylinder(stick_length / 2.0, 0.05)
            .density(0.5)
            .friction(0.2)
            .build();
        collider_set.insert_with_parent(stick_collider, stick_handle, &mut rigid_body_set);
        
        // Joint sphérique entre le chariot et le bâton (point de pivot)
        let mut impulse_joint_set = ImpulseJointSet::new();
        let joint = SphericalJointBuilder::new()
            .local_anchor1(point![0.0, 0.2, 0.0]) // Haut du chariot
            .local_anchor2(point![0.0, -stick_length / 2.0, 0.0]) // Base du bâton
            .build();
        impulse_joint_set.insert(cart_handle, stick_handle, joint, true);
        
        Self {
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set,
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            rigid_body_set,
            collider_set,
            cart_handle,
            stick_handle,
            duration,
            time_step: 1.0 / 60.0,
        }
    }

    pub fn run(&mut self) -> Vec<f64> {
        let mut data_stream = Vec::new();
        let mut rng = rand::thread_rng();
        let num_steps = (self.duration / self.time_step) as usize;
    
        // Paramètres d'intégration haute précision
        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.dt = self.time_step as Real;
        integration_parameters.num_solver_iterations = std::num::NonZeroUsize::new(12).unwrap();

        for step in 0..num_steps {
            // 1. Détermination de la force appliquée (u)
            // On enregistre la force brute pour que le moteur FOL puisse induire F=ma
            let mut applied_force = 0.0;
        
        if rng.gen_bool(0.05) { // Perturbation aléatoire
            applied_force = rng.gen_range(-20.0..20.0);
            if let Some(cart) = self.rigid_body_set.get_mut(self.cart_handle) {
                cart.apply_impulse(vector![applied_force, 0.0, 0.0], true);
            }
        }

        // 2. Step de physique
        self.physics_pipeline.step(
            &vector![0.0, -9.81, 0.0],
            &integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &(),
            &(),
        );

        // 3. Extraction de l'Espace des Phases (Phase Space)
        if let (Some(cart), Some(stick)) = (
            self.rigid_body_set.get(self.cart_handle),
            self.rigid_body_set.get(self.stick_handle)
        ) {
            let cart_x = cart.translation().x as f64;
            let cart_v = cart.linvel().x as f64;
            let stick_angle = stick.rotation().angle() as f64;
            let stick_av = stick.angvel().x as f64; // Vitesse angulaire sur l'axe X (pivot)

            // Structure du vecteur : [x, theta, dx/dt, dtheta/dt, F]
            // Cette structure d=5 est optimale pour l'intégrale itérée de Chen
            data_stream.push(cart_x);      // Dimension 1
            data_stream.push(stick_angle); // Dimension 2
            data_stream.push(cart_v);      // Dimension 3
            data_stream.push(stick_av);     // Dimension 4
            data_stream.push(applied_force); // Dimension 5 (Contrôle/Causalité)
            }
        }
    
        data_stream
    }
    
    /// Vérifie si le bâton est tombé (angle > 45 degrés)
    pub fn has_fallen(&self) -> bool {
        if let Some(stick) = self.rigid_body_set.get(self.stick_handle) {
            let angle = stick.rotation().angle().to_degrees();
            angle.abs() > 45.0
        } else {
            false
        }
    }
}