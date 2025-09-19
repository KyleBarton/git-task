use crate::Task;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
        match new.get_comments() {
            None => {
                // new comments are none old comments are some, so a delete must have occurred
                actions.push(TaskAction::DeleteComment);
            }
            Some(new_comments) => {
                match old.get_comments() {
                    None => {
                        actions.push(TaskAction::AddComment);
                    }
                    Some(old_comments) => {
                        if new_comments.len() < old_comments.len() {
                            actions.push(TaskAction::DeleteComment);
                        }
                    }
                }
            }
        }
    }
    let old_props = props_minus_status(old);
    let new_props = props_minus_status(new);
    if new_props != old_props {
        if new_props.len() > old_props.len() {
            actions.push(TaskAction::SetProperty);
        } else if new_props.len() < old_props.len() {
            actions.push(TaskAction::DeleteProperty);
        } else {
            actions.push(TaskAction::EditProperty);
        }
    }
    if old.labels != new.labels {
        match &new.labels {
            None => {
                // new comments are none old comments are some, so a delete must have occurred
                actions.push(TaskAction::DeleteLabel);
            }
            Some(new_labels) => {
                match &old.labels {
                    None => {
                        actions.push(TaskAction::AddLabel);
                    }
                    Some(old_labels) => {
                        if new_labels.len() < old_labels.len() {
                            actions.push(TaskAction::DeleteLabel);
                        } else {
                            // Notably doesn't include label updates, which are not supported today
                            actions.push(TaskAction::AddLabel);
                        }
                    }
                }
            }
        }
    }
    TaskChangeset::new(actions)
}

fn props_minus_status(task: &Task) -> HashMap<&String, &String> {
    task.props.iter().filter(|(k, _v)| { *k != "status"}).collect()
}