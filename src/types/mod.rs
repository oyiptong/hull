#[derive(Serialize, Deserialize, Debug)]
pub struct Event<T> {
    pub event_name: String,
    pub event_data: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommandRunTime {
    pub cmd: String,
    pub args: Vec<String>,
    pub run: f64,
    pub created_at: i64,
    pub status_code: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CurrentTime {
    pub created_at: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventsPayload<T> {
    pub events: Vec<Event<T>>,
}
