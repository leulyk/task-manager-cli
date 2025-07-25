use std::collections::HashMap;

pub struct Epic {
    name: String,
    description: String,
    status: Status,
    stories: Vec<u32>
}

impl Epic {
    fn new(name: &str, description: &str) -> Self {
       Self {
            name: name.to_owned(),
            description: description.to_owned(),
            status: Status::Open,
            stories: vec![]
        }
    }
}

pub enum Status {
    Open,
    InProgress,
    Resolved,
    Closed,
}

pub struct Story {
    name: String,
    description: String,
    status: Status    
}

impl Story {
    fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            status: Status::Open
        }
    }
}

pub struct DBState {
    last_item_id: u32,
    epics: HashMap<u32, Epic>,
    stories: HashMap<u32, Story>
}
