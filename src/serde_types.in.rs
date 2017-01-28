#[derive(Serialize, Deserialize, Debug)]
struct Event<T> {
    event_name: String,
    event_data: T,
}

#[derive(Serialize, Deserialize, Debug)]
struct CommandRunTime {
    cmd: String,
    args: Vec<String>,
    run: f64,
    created_at: i64,
    status_code: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct CurrentTime {
    created_at: i64,
}
