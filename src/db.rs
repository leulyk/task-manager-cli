use crate::models::DBState;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Story, Epic};
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
            file_path: file.path().to_str().unwrap().to_owned()
        };

        assert_eq!(db.read_db().is_ok(), true);
    }

    #[test]
    fn write_db_should_work() {
        let story = Story::new("story_1", "story_description");
        let mut epic = Epic::new("epic_1", "epic_description");

        let mut stories = HashMap::new();
        stories.insert(2, story) ;
        epic.stories.push(2);

        let mut epics = HashMap::new();
        epics.insert(1, epic);

        let db_state = DBState {
            last_item_id: 2,
            stories,
            epics
        };

        let file = NamedTempFile::new().unwrap();
        let db = JSONFileDatabase {
            file_path: file.path().to_str().unwrap().to_owned()
        };

        assert_eq!(db.write_db(&db_state).is_ok(), true);
        let content = db.read_db().unwrap();

        assert_eq!(db_state, content);
    }
}
