struct Stick {
  width: f64,
  height: f64,
  weight: f64,
}

struct Chariot {
  width: f64,
  height: f64,
  weight: f64,
}

pub struct BalanceStickAnimation {
  stick: Stick,
  chariot: Chariot,
}

impl BalanceStickAnimation {
  pub fn start(&self) -> Vec<f64> {
    self.construct();
    //
  }

  pub fn construct() {
    
  }
}
