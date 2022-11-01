use anyhow::{Context, Result};

use crate::models::{DBState, Epic, Status, Story};

use nanoid::nanoid;

pub trait Database {
    fn read_db(&self) -> Result<DBState, anyhow::Error>;
    fn write_db(&self, db_state: &DBState) -> Result<()>;
}

struct JSONFileDatabase {
    pub file_path: String,
}

impl Database for JSONFileDatabase {
    fn read_db(&self) -> Result<DBState, anyhow::Error> {
        // Read the file
        let file_contents = std::fs::read_to_string(&self.file_path)
            .with_context(|| format!("Failed to read from file system."))?;
        // Deserialize the file contents into a DBState
        let db_state: DBState = serde_json::from_str(&file_contents)
            .with_context(|| "Failed to write current state to memory.")?;
        // Return the DBState
        Ok(db_state)
    }

    fn write_db(&self, db_state: &DBState) -> Result<(), anyhow::Error> {
        // Serialize db_state to json and store it in self.file_path
        let file_contents = serde_json::to_string_pretty(&db_state)
            .with_context(|| "Failed to write current state to memory.")?;
        // Write to file
        std::fs::write(&self.file_path, file_contents).map_err(|e| e.into())
    }
}

pub struct JiraDatabase {
    pub database: Box<dyn Database>,
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

    pub fn create_epic(&self, epic: Epic) -> Result<String> {
        // Grab a mutable reference to the database
        let mut db_state = self.read_db()?;
        // Create a new epic
        let epic = Epic::new(epic.name, epic.description);
        // Generate a new id
        let id = nanoid!(6);
        // Add the epic to the database
        db_state.epics.insert(id.clone(), epic);
        // Add last_item_id to the database
        db_state.last_item_id = id.clone();
        // Write the database to disk
        self.database.write_db(&db_state)?;
        // Return the id of the new epic
        Ok(id)
    }

    pub fn create_story(&self, story: Story, epic_id: &String) -> Result<String, anyhow::Error> {
        // Grab a mutable reference to the database
        let mut db_state = self
            .read_db()
            .with_context(|| format!("Failed to read database when creating story."))?;

        // Create a new story
        let story = Story::new(story.name, story.description);

        // Check if the epic exists
        if !db_state.epics.contains_key(epic_id) {
            return Err(anyhow::anyhow!("Epic with id {} does not exist.", epic_id));
        }

        // Generate story id
        let id = nanoid!(6);

        // Add last_item_id to the database
        db_state.last_item_id = id.clone();

        // Add the story to the database
        db_state.stories.insert(id.clone(), story);

        // Add story to corresponding epic
        db_state
            .epics
            .get_mut(epic_id)
            .unwrap()
            .stories
            .push(id.clone());

        // Write the database to disk
        self.database.write_db(&db_state)?;

        // Return the id of the new story
        Ok(id)
    }

    pub fn delete_epic(&self, epic_id: &String) -> Result<(), anyhow::Error> {
        // Grab a mutable reference to the database
        let mut db_state = self.read_db().with_context(|| "Failed to read database.")?;
        // Grab a mutable reference to the epic
        let epic = db_state
            .epics
            .get_mut(epic_id)
            .with_context(|| format!("Epic with id {} does not exist.", epic_id))?;
        // Delete all stories associated with the epic
        for story_id in epic.stories.iter() {
            db_state.stories.remove(story_id);
        }
        // Delete the epic
        db_state.epics.remove(epic_id);
        // Set epic ID as the last item id
        db_state.last_item_id = epic_id.to_string();
        // Write the database to disk
        self.database.write_db(&db_state)?;
        // Return Ok
        Ok(())
    }

    pub fn delete_story(&self, epic_id: &String, story_id: &String) -> Result<()> {
        // Grab a mutable reference to the database
        let mut db_state = self.read_db()?;

        // Confirm that the story actually exists
        if !db_state.stories.contains_key(story_id) {
            return Err(anyhow::anyhow!(
                "Story with id {} does not exist.",
                story_id
            ));
        }

        // Grab a mutable reference to the epic
        let epic = db_state
            .epics
            .get_mut(epic_id)
            .with_context(|| format!("Epic with id {} does not exist.", epic_id))?;

        // Iterate over epic stories and remove the story
        epic.stories.retain(|id| id != story_id);

        // Find the corresponding story and remove it
        db_state.stories.remove(story_id);

        // Set story ID as the last item id
        db_state.last_item_id = story_id.to_string();

        // Write the database to disk
        self.database.write_db(&db_state)?;

        // Return Ok
        Ok(())
    }

    pub fn update_epic_status(&self, epic_id: &String, status: Status) -> Result<()> {
        // Grab database
        let mut db_state = self.read_db()?;
        // Grab a mutable reference to the epic
        let epic = db_state
            .epics
            .get_mut(epic_id)
            .with_context(|| format!("Epic with id {} does not exist.", epic_id))?;
        // Update epic status
        epic.status = status;
        // Write the database to disk
        self.database.write_db(&db_state)?;
        // Return Ok
        Ok(())
    }

    pub fn update_story_status(&self, story_id: &String, status: Status) -> Result<()> {
        // Grab database
        let mut db_state = self.read_db()?;
        // Grab a mutable reference to the epic
        let story = db_state
            .stories
            .get_mut(story_id)
            .with_context(|| format!("Story with id {} does not exist.", story_id))?;
        // Update story status
        story.status = status;
        // Write the database to disk
        self.database.write_db(&db_state)?;
        // Return Ok
        Ok(())
    }

    pub fn get_epic(&self, epic_id: &String) -> Result<Epic> {
        // Grab database
        let db_state = self.read_db()?;
        // Grab a mutable reference to the epic
        let epic = db_state
            .epics
            .get(epic_id)
            .with_context(|| format!("Epic with id {} does not exist.", epic_id))?;
        // Return Ok
        Ok(epic.clone())
    }

    pub fn get_epic_story(&self, epic_id: &String, story_id: &String) -> Result<Story> {
        // Grab database
        let db_state = self.read_db()?;
        // Grab a mutable reference to the epic
        let epic = db_state
            .epics
            .get(epic_id)
            .with_context(|| format!("Epic with id {} does not exist.", epic_id))?;
        // Grab a mutable reference to the story
        let story = epic
            .stories
            .iter()
            .find(|id| id == &story_id)
            .with_context(|| format!("Story with id {} does not exist.", story_id))?;
        // Return Ok
        Ok(db_state.stories.get(story).unwrap().clone())
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
                    last_item_id: "0".to_string(),
                    epics: HashMap::new(),
                    stories: HashMap::new(),
                }),
            }
        }
    }

    impl Database for MockDB {
        fn read_db(&self) -> Result<DBState> {
            let state = self.last_written_state.borrow().clone();
            Ok(state)
        }

        fn write_db(&self, db_state: &DBState) -> Result<()> {
            let latest_state = &self.last_written_state;
            *latest_state.borrow_mut() = db_state.clone();
            Ok(())
        }
    }

    pub fn arrange_test() -> (JiraDatabase, String, String) {
        // Arrange db and data
        let mock = Box::new(MockDB::new());
        let db = JiraDatabase { database: mock };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        // Add data to db
        let result = db.create_epic(epic);
        let epic_id = result.unwrap();
        let result = db.create_story(story, &epic_id);
        let story_id = result.unwrap();

        (db, epic_id, story_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::db::test_utils::arrange_test;
    use crate::models::{DBState, Epic, Story};
    use nanoid::nanoid;

    use super::test_utils::MockDB;
    use super::*;

    #[test]
    fn create_epic_should_work() {
        // Arrange
        let mock = Box::new(MockDB::new());
        let db = JiraDatabase { database: mock };
        let epic = Epic::new("An Epic".to_owned(), "Description".to_owned());

        // Act
        let result = db.create_epic(epic.clone());

        // Assert part 1
        assert_eq!(result.is_ok(), true);

        // Arrange part 2
        let epic_id = result.unwrap();
        let db_state = db.read_db().unwrap();

        // Assert
        assert_eq!(db_state.epics.get(&epic_id), Some(&epic));
        assert_eq!(&db_state.last_item_id, &epic_id);
    }

    #[test]
    fn create_story_should_error_if_invalid_epic_id() {
        // Arrange
        let mock = Box::new(MockDB::new());
        let db = JiraDatabase { database: mock };
        let story = Story::new("".to_owned(), "".to_owned());
        let non_existent_epic_id = nanoid!(6);

        // Act
        let result = db.create_story(story, &non_existent_epic_id);

        // Assert
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn create_story_should_work() {
        // Arrange test
        let (db, epic_id, story_id) = arrange_test();
        let story = Story::new("".to_owned(), "".to_owned());

        // Arrange for reading the DB assertion
        let db_state = db.read_db().unwrap();
        let contains_story = db_state
            .epics
            .get(&epic_id)
            .unwrap()
            .stories
            .contains(&story_id);

        // Assert
        assert_eq!(db_state.last_item_id, story_id);
        assert_eq!(contains_story, true);
        assert_eq!(db_state.stories.get(&story_id), Some(&story));
    }

    #[test]
    fn delete_epic_should_error_if_invalid_epic_id() {
        // Arrange
        let mock = Box::new(MockDB::new());
        let db = JiraDatabase { database: mock };
        let non_existent_epic_id = nanoid!(6);

        // Act
        let result = db.delete_epic(&non_existent_epic_id);

        // Assert
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn delete_epic_should_work() {
        // Arrange test
        let (db, epic_id, story_id) = arrange_test();

        // Act
        let result = db.delete_epic(&epic_id);
        let db_state = db.read_db().unwrap();

        // Assert
        assert_eq!(result.is_ok(), true);
        assert_eq!(db_state.last_item_id, epic_id);
        assert_eq!(db_state.epics.get(&epic_id), None);
        assert_eq!(db_state.stories.get(&story_id), None);
    }

    #[test]
    fn delete_story_should_error_if_invalid_epic_id() {
        // Arrange test
        let (db, _epic_id, story_id) = arrange_test();

        // Generate a non-existent epic id
        let non_existent_epic_id = nanoid!(6);

        // Act
        let result = db.delete_story(&non_existent_epic_id, &story_id);

        // Assert
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn delete_story_should_error_if_story_not_found_in_epic() {
        // Arrange test
        let (db, epic_id, _story_id) = arrange_test();

        // Generate a non-existent story id
        let non_existent_story_id = nanoid!(6);

        // Act
        let result = db.delete_story(&epic_id, &non_existent_story_id);

        // Assert
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn delete_story_should_work() {
        // Arrange test
        let (db, epic_id, story_id) = arrange_test();

        // Act
        let result = db.delete_story(&epic_id, &story_id);
        let db_state = db.read_db().unwrap();
        let contains_stories = db_state
            .epics
            .get(&epic_id)
            .unwrap()
            .stories
            .contains(&story_id);

        // Assert
        assert_eq!(result.is_ok(), true);
        assert_eq!(db_state.last_item_id, story_id);
        assert_eq!(contains_stories, false);
        assert_eq!(db_state.stories.get(&story_id), None);
    }

    #[test]
    fn update_epic_status_should_error_if_invalid_epic_id() {
        // Arrange
        let mock = Box::new(MockDB::new());
        let db = JiraDatabase { database: mock };
        let non_existent_epic_id = nanoid!(6);

        // Act
        let result = db.update_epic_status(&non_existent_epic_id, Status::Closed);

        // Assert
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn update_epic_status_should_work() {
        // Arrange test
        let (db, epic_id, _story_id) = arrange_test();

        // Act
        let result = db.update_epic_status(&epic_id, Status::Closed);
        let db_state = db.read_db().unwrap();

        // Assert
        assert_eq!(result.is_ok(), true);
        assert_eq!(db_state.epics.get(&epic_id).unwrap().status, Status::Closed);
    }

    #[test]
    fn update_story_status_should_error_if_invalid_story_id() {
        // Arrange
        let mock = Box::new(MockDB::new());
        let db = JiraDatabase { database: mock };
        let non_existent_story_id = nanoid!(6);

        // Act
        let result = db.update_story_status(&non_existent_story_id, Status::Closed);

        // Assert
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn update_story_status_should_work() {
        // Arrange test
        let (db, _epic_id, story_id) = arrange_test();

        // Act
        let result = db.update_story_status(&story_id, Status::Closed);
        let db_state = db.read_db().unwrap();
        let new_status = &db_state.stories.get(&story_id).unwrap().status;

        // Assert
        assert_eq!(result.is_ok(), true);
        assert_eq!(*new_status, Status::Closed);
    }

    mod database {
        use std::collections::HashMap;
        use std::fs::remove_file;
        use std::io::Write;

        use super::*;

        #[test]
        fn read_db_should_fail_with_invalid_path() {
            let db = JSONFileDatabase {
                file_path: "INVALID_PATH".to_owned(),
            };
            assert_eq!(db.read_db().is_err(), true);
        }

        #[test]
        fn read_db_should_fail_with_invalid_json() {
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();

            let file_contents = r#"{ "last_item_id": 0 epics: {} stories {} }"#;
            write!(tmpfile, "{}", file_contents).unwrap();

            let file_path = "./data/read_db_should_fail_with_invalid_json.json".to_owned();

            let path = tmpfile.into_temp_path();
            path.persist(&file_path).unwrap();

            let db = JSONFileDatabase {
                file_path: file_path.clone(),
            };

            let result = db.read_db();

            remove_file(file_path).unwrap();

            assert_eq!(result.is_err(), true);
        }

        #[test]
        fn read_db_should_parse_json_file() {
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();

            let file_contents = r#"{ "last_item_id": "0", "epics": {}, "stories": {} }"#;
            write!(tmpfile, "{}", file_contents).unwrap();

            let file_path = "./data/read_db_should_parse_json_file.json".to_owned();

            let path = tmpfile.into_temp_path();
            path.persist(&file_path).unwrap();

            let db = JSONFileDatabase {
                file_path: file_path.clone(),
            };

            let result = db.read_db();

            remove_file(file_path).unwrap();

            assert_eq!(result.is_ok(), true);
        }

        #[test]
        fn write_db_should_work() {
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();

            let file_contents = r#"{ "last_item_id": "0", "epics": {}, "stories": {} }"#;
            write!(tmpfile, "{}", file_contents).unwrap();

            let file_path = "./data/write_db_should_work.json".to_owned();

            let path = tmpfile.into_temp_path();
            path.persist(&file_path).unwrap();

            let db = JSONFileDatabase {
                file_path: file_path.clone(),
            };

            let story = Story {
                name: "epic 1".to_owned(),
                description: "epic 1".to_owned(),
                status: Status::Open,
            };
            let epic = Epic {
                name: "epic 1".to_owned(),
                description: "epic 1".to_owned(),
                status: Status::Open,
                stories: vec!["2".to_owned()],
            };

            let mut stories = HashMap::new();
            stories.insert("2".to_owned(), story);

            let mut epics = HashMap::new();
            epics.insert("1".to_owned(), epic);

            let state = DBState {
                last_item_id: "1".to_owned(),
                epics,
                stories,
            };

            let write_result = db.write_db(&state);
            let read_result = db.read_db().unwrap();

            remove_file(file_path).unwrap();

            assert_eq!(write_result.is_ok(), true);
            assert_eq!(read_result, state);
        }
    }
}
