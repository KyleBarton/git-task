use serde::{Deserialize, Serialize};
use crate::Task;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum TaskAction {
    TaskCreate,
    UpdateStatus,
    SetProperty,
    EditProperty,
    DeleteProperty,
    SearchReplaceProperty,
    AddComment,
    DeleteComment,
    AddLabel,
    UpdateLabel,
    DeleteLabel,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TaskChangeset {
    actions: Vec<TaskAction>,
}
impl TaskChangeset {
    pub fn new(actions: Vec<TaskAction>) -> Self {
        Self { actions }
    }
}

pub fn get_changeset(old: &Task, new: &Task) -> TaskChangeset {

    let mut actions = vec![];
    if old.get_property("status") != new.get_property("status") {
        actions.push(TaskAction::UpdateStatus);
    }
    if old.get_comments() != new.get_comments() {
        actions.push(TaskAction::AddComment);
    }
    TaskChangeset::new(actions)
}