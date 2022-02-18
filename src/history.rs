use chrono::NaiveDateTime;

#[derive(serde::Serialize, Debug)]
pub struct History {
    scans: Vec<Scan>,
}

#[derive(serde::Serialize, Debug)]
pub struct Scan {
    time: NaiveDateTime,
    events: Vec<Event>,
}

#[derive(serde::Serialize, Debug)]
pub struct Event {
    relative_full_file_name: String,
    time: NaiveDateTime,
    file_size: usize,
    type_: EventType,
}

#[derive(serde::Serialize, Debug)]
pub enum EventType {
    Add,
    Edit,
    Delete,
    Gen,
}

impl History {
    pub fn read() -> Self {

    }
}
