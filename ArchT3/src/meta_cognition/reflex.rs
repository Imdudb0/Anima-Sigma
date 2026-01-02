#[derive(Clone)]
pub struct ReflexMetrics {
    pub actions_count: u32,
    pub average_response_time_ms: f64,
    pub errors: Vec<String>,
}

#[derive(Clone)]
pub struct ReflexConfig {
    pub reaction_threshold: f64,
    pub pattern: String,
    pub cooldown_ms: u64,
}