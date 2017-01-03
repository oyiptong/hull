#[derive(Serialize, Deserialize, Debug)]
struct Event<T> {
    event_name: String,
    event_data: T,
}

#[derive(Serialize, Deserialize, Debug)]
struct PerfTimings {
    cmd: String,
    boot: f64,
    run: f64,
    created_at: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct CurrentTime {
    created_at: i64,
}
