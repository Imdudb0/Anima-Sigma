use ArchT3::{
BalanceStickAnimation,
};

fn main() {
    let mut simulator = BalanceStickAnimation::new(30.0);
    let start_time = Instant::now();
   
    loop {
        let now = Instant::now();
        let elapsed_time = now.duration_since(start_time).as_secs_f64();

        let dx = simulator.run();
        println!("dx: {:?}", dx);

        if elapsed_time >= 30.0 {
            break;
        }
    }
}