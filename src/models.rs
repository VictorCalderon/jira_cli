#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    NavigateToEpicDetail { epic_id: String },
    NavigateToStoryDetail { epic_id: String, story_id: String },
    NavigateToPreviousPage,
    CreateEpic,
    UpdateEpicStatus { epic_id: String },
    DeleteEpic { epic_id: String },
    CreateStory { epic_id: String },
    UpdateStoryStatus { story_id: String },
    DeleteStory { epic_id: String, story_id: String },
    Exit,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Open => write!(f, "OPEN"),
            Status::InProgress => write!(f, "IN PROGRESS"),
            Status::Resolved => write!(f, "RESOLVED"),
            Status::Closed => write!(f, "CLOSED"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Status {
    InProgress,
    Closed,
    Open,
    Resolved,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Epic {
    pub name: String,
    pub description: String,
    pub status: Status,
    pub stories: Vec<String>,
}

impl Epic {
    pub fn new(name: String, description: String) -> Self {
        return Self {
            name,
            description,
            status: Status::Open,
            stories: Vec::new(),
        };
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Story {
    pub name: String,
    pub description: String,
    pub status: Status,
}

impl Story {
    pub fn new(name: String, description: String) -> Self {
        return Self {
            name,
            description,
            status: Status::Open,
        };
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DBState {
    pub epics: HashMap<String, Epic>,
    pub stories: HashMap<String, Story>,
    pub last_item_id: String,
}
