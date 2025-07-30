use crate::models::{DBState, Epic, Status, Story};
use anyhow::Result;
use std::fs;

trait Database {
    fn read_db(&self) -> Result<DBState>;
    fn write_db(&self, db_state: &DBState) -> Result<()>;
}

struct JSONFileDatabase {
    pub file_path: String,
}

impl Database for JSONFileDatabase {
    fn read_db(&self) -> Result<DBState> {
        let content = fs::read_to_string(&self.file_path)?;
        let db_state: DBState = serde_json::from_str(&content)?;

        Ok(db_state)
    }

    fn write_db(&self, db_state: &DBState) -> Result<()> {
        let content = serde_json::to_string(db_state)?;
        fs::write(&self.file_path, content)?;

        Ok(())
    }
}

pub struct JiraDatabase {
    database: Box<dyn Database>,
}

impl JiraDatabase {
    pub fn new(file_path: String) -> Self {
        Self {
            database: Box::new(JSONFileDatabase { file_path }),
        }
    }

    pub fn read_db(&self) -> Result<DBState> {
        self.database.read_db()
    }

    pub fn create_epic(&self, epic: Epic) -> Result<u32> {
        todo!()
    }

    pub fn create_story(&self, story: Story, epic_id: u32) -> Result<u32> {
        todo!()
    }

    pub fn delete_epic(&self, epic_id: u32) -> Result<()> {
        todo!()
    }

    pub fn delete_story(&self, epic_id: u32, story_id: u32) -> Result<()> {
        todo!()
    }

    pub fn update_epic_status(&self, epic_id: u32, status: Status) -> Result<()> {
        todo!()
    }

    pub fn update_story_status(&self, story_id: u32, status: Status) -> Result<()> {
        todo!()
    }
}

mod test_utils {
    use super::*;
    use std::cell::RefCell;

    pub struct MockDb {
        last_written_state: RefCell<DBState>,
    }

    impl Database for MockDb {
        fn read_db(&self) -> Result<DBState> {
            todo!()
        }

        fn write_db(&self, db_state: &DBState) -> Result<()> {
            todo!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod database {
        use super::*;
        use std::{collections::HashMap, io::Write};
        use tempfile::NamedTempFile;

        #[test]
        fn read_db_should_fail_on_invalid_path() {
            let db = JSONFileDatabase {
                file_path: "wrong_path".to_owned(),
            };
            assert_eq!(db.read_db().is_err(), true);
        }

        #[test]
        fn read_db_should_fail_on_invalid_json() {
            let mut file = NamedTempFile::new().unwrap();

            let invalid_json = r#"{ "last_item_id": 0 "epics": {} stories: {} }"#;
            file.write(invalid_json.as_bytes()).unwrap();

            let db = JSONFileDatabase {
                file_path: file.path().to_str().unwrap().to_owned(),
            };

            let result = db.read_db();

            assert_eq!(result.is_err(), true);
        }

        #[test]
        fn read_db_should_parse_valid_json() {
            let mut file = NamedTempFile::new().unwrap();

            let valid_json = r#"{ "last_item_id": 0, "epics": {}, "stories": {} }"#;
            file.write(valid_json.as_bytes()).unwrap();

            let db = JSONFileDatabase {
                file_path: file.path().to_str().unwrap().to_owned(),
            };

            assert_eq!(db.read_db().is_ok(), true);
        }

        #[test]
        fn write_db_should_work() {
            let story = Story::new("story_1", "story_description");
            let mut epic = Epic::new("epic_1", "epic_description");

            let mut stories = HashMap::new();
            stories.insert(2, story);
            epic.stories.push(2);

            let mut epics = HashMap::new();
            epics.insert(1, epic);

            let db_state = DBState {
                last_item_id: 2,
                stories,
                epics,
            };

            let file = NamedTempFile::new().unwrap();
            let db = JSONFileDatabase {
                file_path: file.path().to_str().unwrap().to_owned(),
            };

            assert_eq!(db.write_db(&db_state).is_ok(), true);
            let content = db.read_db().unwrap();

            assert_eq!(db_state, content);
        }
    }
}
