struct PVC {
  outer_diameter: f64,
  height_mm: f64,
  weight_g: f64,
  wall_thickness: f64,
  position: f64,
}

struct Chariot {
  width: f64,
  height: f64,
  weight_kg: f64,
  position: f64,
}

pub struct BalanceStickAnimation {
  pvc: PVC,
  chariot: Chariot,
}

impl BalanceStickAnimation {
  pub fn start(&self, duration: f64, ) -> Vec<f64> {
    self.construct();
    //
  }

  pub fn construct() {
    self.pvc = PVC::new(1500, 114.3, 6.0198);
    self.chariot = Chariot::new();
  }
}

impl PVC {
    fn new(height_mm: f64, outer_diameter: f64, wall_thickness: f64) -> Self {
        let rho = 1.4; // g/cm³
        let r_outer = outer_diameter / 2.0;
        let r_inner = r_outer - wall_thickness;

        let volume_cm3 = std::f64::consts::PI * (r_outer.powi(2) - r_inner.powi(2)) * height_mm / 1000.0;
        let weight_g = rho * volume_cm3;

        Self {
            outer_diameter,
            height_mm,
            weight_g,
            wall_thickness,
            position: 0.0,
        }
    }
}

impl Chariot {
  pub fn new() -> Self {
    let weight_g;
  }
}
