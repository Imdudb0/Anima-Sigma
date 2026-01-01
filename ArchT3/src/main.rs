use ArchT3::{
BalanceStickAnimation,
};

fn main() {
    let mut simulation = BalanceStickAnimation::new(30.0);
    loop {
       let dx = simulation.run_for_signature();
       println!("dx: {:?}", dx);
    }
}