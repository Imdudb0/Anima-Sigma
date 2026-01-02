fn main() {
    loop {
        let now = Instant::now();
        let elapsed_time = now.duration_since(start_time).as_secs_f64();

        

        if elapsed_time >= 30.0 {
            break;
        }
    }
}