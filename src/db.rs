use crate::models::{DBState, Epic, Status, Story};
use anyhow::{anyhow, Result};
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
        let mut db = self.read_db()?;

        let new_id = db.last_item_id + 1;
        db.epics.insert(new_id, epic);
        db.last_item_id = new_id;

        self.database.write_db(&db)?;

        Ok(new_id)
    }

    pub fn create_story(&self, story: Story, epic_id: u32) -> Result<u32> {
        let mut db = self.read_db()?;
        let new_id = db.last_item_id + 1;

        let epic = db
            .epics
            .get_mut(&epic_id)
            .ok_or(anyhow!("Couldn't find epic..."))?;

        db.stories.insert(new_id, story);
        epic.stories.push(new_id);
        db.last_item_id = new_id;

        self.database.write_db(&db)?;

        Ok(new_id)
    }

    pub fn delete_epic(&self, epic_id: u32) -> Result<()> {
        let mut db = self.read_db()?;

        let epic = db.epics.get(&epic_id).ok_or(anyhow!("Epic not found..."))?;

        for story in &epic.stories {
            db.stories.remove(&story);
        }

        db.epics
            .remove(&epic_id)
            .ok_or(anyhow!("Epic not found..."))?;

        self.database.write_db(&db)?;

        Ok(())
    }

    pub fn delete_story(&self, epic_id: u32, story_id: u32) -> Result<()> {
        let mut db = self.read_db()?;

        let epic = db
            .epics
            .get_mut(&epic_id)
            .ok_or(anyhow!("Epic not found..."))?;

        let story_index = epic
            .stories
            .iter()
            .position(|id| id == &story_id)
            .ok_or(anyhow!("Story not found in epic..."))?;

        epic.stories.remove(story_index);
        db.stories.remove(&story_id);

        self.database.write_db(&db)?;

        Ok(())
    }

    pub fn update_epic_status(&self, epic_id: u32, status: Status) -> Result<()> {
        let mut db = self.read_db()?;

        let epic = db
            .epics
            .get_mut(&epic_id)
            .ok_or(anyhow!("Couldn't find epic..."))?;

        epic.status = status;
        self.database.write_db(&db)?;

        Ok(())
    }

    pub fn update_story_status(&self, story_id: u32, status: Status) -> Result<()> {
        let mut db = self.read_db()?;

        db.stories
            .get_mut(&story_id)
            .ok_or(anyhow!("Couldn't find story..."))?
            .status = status;

        self.database.write_db(&db)?;

        Ok(())
    }
}

pub mod test_utils {
    use super::*;
    use std::{cell::RefCell, collections::HashMap};

    pub struct MockDB {
        last_written_state: RefCell<DBState>,
    }

    impl MockDB {
        pub fn new() -> Self {
            Self {
                last_written_state: RefCell::new(DBState {
                    last_item_id: 0,
                    epics: HashMap::new(),
                    stories: HashMap::new(),
                }),
            }
        }
    }

    impl Database for MockDB {
        fn read_db(&self) -> Result<DBState> {
            Ok(self.last_written_state.borrow().clone())
        }

        fn write_db(&self, db_state: &DBState) -> Result<()> {
            *self.last_written_state.borrow_mut() = db_state.clone();

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::MockDB;
    use super::*;

    #[test]
    fn create_epic_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic.clone());

        assert_eq!(result.is_ok(), true);

        let id = result.unwrap();
        let db_state = db.read_db().unwrap();

        let expected_id = 1;

        assert_eq!(id, expected_id);
        assert_eq!(db_state.last_item_id, expected_id);
        assert_eq!(db_state.epics.get(&id), Some(&epic));
    }

    #[test]
    fn create_story_should_error_if_invalid_epic_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let story = Story::new("".to_owned(), "".to_owned());

        let non_existent_epic_id = 999;

        let result = db.create_story(story, non_existent_epic_id);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn create_story_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story.clone(), epic_id);
        assert_eq!(result.is_ok(), true);

        let id = result.unwrap();
        let db_state = db.read_db().unwrap();

        let expected_id = 2;

        assert_eq!(id, expected_id);
        assert_eq!(db_state.last_item_id, expected_id);
        assert_eq!(
            db_state.epics.get(&epic_id).unwrap().stories.contains(&id),
            true
        );
        assert_eq!(db_state.stories.get(&id), Some(&story));
    }

    #[test]
    fn delete_epic_should_error_if_invalid_epic_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };

        let non_existent_epic_id = 999;

        let result = db.delete_epic(non_existent_epic_id);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn delete_epic_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert_eq!(result.is_ok(), true);

        let story_id = result.unwrap();

        let result = db.delete_epic(epic_id);
        assert_eq!(result.is_ok(), true);

        let db_state = db.read_db().unwrap();

        let expected_last_id = 2;

        assert_eq!(db_state.last_item_id, expected_last_id);
        assert_eq!(db_state.epics.get(&epic_id), None);
        assert_eq!(db_state.stories.get(&story_id), None);
    }

    #[test]
    fn delete_story_should_error_if_invalid_epic_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert_eq!(result.is_ok(), true);

        let story_id = result.unwrap();

        let non_existent_epic_id = 999;

        let result = db.delete_story(non_existent_epic_id, story_id);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn delete_story_should_error_if_story_not_found_in_epic() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert_eq!(result.is_ok(), true);

        let non_existent_story_id = 999;

        let result = db.delete_story(epic_id, non_existent_story_id);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn delete_story_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert_eq!(result.is_ok(), true);

        let story_id = result.unwrap();

        let result = db.delete_story(epic_id, story_id);
        assert_eq!(result.is_ok(), true);

        let db_state = db.read_db().unwrap();

        let expected_last_id = 2;

        assert_eq!(db_state.last_item_id, expected_last_id);
        assert_eq!(
            db_state
                .epics
                .get(&epic_id)
                .unwrap()
                .stories
                .contains(&story_id),
            false
        );
        assert_eq!(db_state.stories.get(&story_id), None);
    }

    #[test]
    fn update_epic_status_should_error_if_invalid_epic_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };

        let non_existent_epic_id = 999;

        let result = db.update_epic_status(non_existent_epic_id, Status::Closed);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn update_epic_status_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);

        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.update_epic_status(epic_id, Status::Closed);

        assert_eq!(result.is_ok(), true);

        let db_state = db.read_db().unwrap();

        assert_eq!(db_state.epics.get(&epic_id).unwrap().status, Status::Closed);
    }

    #[test]
    fn update_story_status_should_error_if_invalid_story_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };

        let non_existent_story_id = 999;

        let result = db.update_story_status(non_existent_story_id, Status::Closed);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn update_story_status_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);

        let story_id = result.unwrap();

        let result = db.update_story_status(story_id, Status::Closed);

        assert_eq!(result.is_ok(), true);

        let db_state = db.read_db().unwrap();

        assert_eq!(
            db_state.stories.get(&story_id).unwrap().status,
            Status::Closed
        );
    }

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
            let story = Story::new("story_1".to_owned(), "story_description".to_owned());
            let mut epic = Epic::new("epic_1".to_owned(), "epic_description".to_owned());

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
