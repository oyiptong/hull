pub struct Metric {
    pub id: i32,
    pub method: String,
    pub name: String,
    pub value: f64,
    pub created_at: String
}

pub struct UploadJob {
    pub id: i32,
    pub metric_id: i32,
    pub name: String,
    pub done: bool,
    pub num_attempts: i32,
    pub created_at: String,
    pub updated_at: String
}
