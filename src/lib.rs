use git2::*;
use serde::{Deserialize, Serialize};
use serde_json;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::ops::Deref;
use std::time::{SystemTime, UNIX_EPOCH};

const NAME: &'static str = "name";
const DESCRIPTION: &'static str = "description";
const STATUS: &'static str = "status";
const CREATED: &'static str = "created";

#[derive(Clone, Serialize, Deserialize)]
pub struct Task {
    id: Option<String>,
    props: HashMap<String, String>,
    comments: Option<Vec<Comment>>,
    labels: Option<Vec<Label>>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    id: Option<String>,
    props: HashMap<String, String>,
    text: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Label {
    name: String,
    color: Option<String>,
    description: Option<String>,
}

#[derive(Clone)]
pub struct TaskContext {
    repository_path: String,
}

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
    // Maybe this just shows before and after task in this case?
    // Ideally, supports backwards compatibility for older tasks.
    UnknownUpdate
}

impl Task {
    pub fn new(name: String, description: String, status: String, author: Option<String>) -> Result<Task, &'static str> {
        if !name.is_empty() && !status.is_empty() {
            Ok(Self::construct_task(name, description, status, author, None))
        } else {
            Err("Name or status is empty")
        }
    }

    pub fn from_properties(id: String, mut props: HashMap<String, String>) -> Result<Task, &'static str> {
        let name = props.get(NAME).unwrap_or(&"".to_owned()).to_owned();
        let status = props.get(STATUS).unwrap_or(&"".to_owned()).to_owned();

        if !name.is_empty() && !status.is_empty() {
            if !props.contains_key("created") {
                props.insert("created".to_string(), get_current_timestamp().to_string());
            }

            Ok(Task{ id: Some(id), props, comments: None, labels: None, })
        } else {
            Err("Name or status is empty")
        }
    }

    fn construct_task(name: String, description: String, status: String, current_user: Option<String>, created: Option<u64>) -> Task {
        let mut props = HashMap::from([
            (NAME.to_owned(), name),
            (DESCRIPTION.to_owned(), description),
            (STATUS.to_owned(), status),
            (CREATED.to_owned(), created.unwrap_or(get_current_timestamp()).to_string()),
        ]);

        if let Some(current_user) = current_user {
            props.insert("author".to_string(), current_user);
        }

        Task {
            id: None,
            props,
            comments: None,
            labels: None,
        }
    }

    pub fn get_id(&self) -> Option<String> {
        match &self.id {
            Some(id) => Some(id.clone()),
            _ => None
        }
    }

    pub fn set_id(&mut self, id: String) {
        self.id = Some(id);
    }

    pub fn get_property(&self, prop: &str) -> Option<&String> {
        self.props.get(prop)
    }

    pub fn get_all_properties(&self) -> &HashMap<String, String> {
        &self.props
    }

    pub fn set_property(&mut self, prop: &str, value: &str) {
        self.props.insert(prop.to_string(), value.to_string());
    }

    pub fn has_property(&self, prop: &str) -> bool {
        self.props.contains_key(prop)
    }

    pub fn delete_property(&mut self, prop: &str) -> bool {
        self.props.remove(prop).is_some()
    }

    pub fn get_comments(&self) -> &Option<Vec<Comment>> {
        &self.comments
    }

    // TODO Can I get rid of the optional ID call here?
    pub fn add_comment(&mut self, id: Option<String>, mut props: HashMap<String, String>, text: String, author: Option<String>) -> Comment {
        if self.comments.is_none() {
            self.comments = Some(vec![]);
        }

        let id = Some(id.unwrap_or_else(|| (self.comments.as_ref().unwrap().len() + 1).to_string()));

        if !props.contains_key("created") {
            props.insert("created".to_string(), get_current_timestamp().to_string());
        }

        if !props.contains_key("author") {
            if let Some(author) = author {
                props.insert("author".to_string(), author);
            }
        }

        let comment = Comment {
            id,
            props,
            text,
        };

        self.comments.as_mut().unwrap().push(comment.clone());

        comment
    }

    pub fn set_comments(&mut self, comments: Vec<Comment>) {
        self.comments = Some(comments);
    }

    pub fn delete_comment(&mut self, id: &String) -> Result<(), String> {
        if self.comments.is_none() {
            return Err("Task has no comments".to_string());
        }

        let index = self.comments.as_ref().unwrap().iter().position(|comment| comment.get_id().unwrap() == id.deref());

        if index.is_none() {
            return Err(format!("Comment ID {id} not found"));
        }

        self.comments.as_mut().unwrap().remove(index.unwrap());

        Ok(())
    }

    pub fn get_labels(&self) -> &Option<Vec<Label>> {
        &self.labels
    }

    pub fn add_label(&mut self, name: String, description: Option<String>, color: Option<String>) -> Label {
        if self.labels.is_none() {
            self.labels = Some(vec![]);
        }

        let label = Label {
            name: name.clone(),
            description,
            color,
        };

        self.labels.as_mut().unwrap().push(label.clone());

        label
    }

    pub fn set_labels(&mut self, labels: Vec<Label>) {
        self.labels = Some(labels);
    }

    pub fn delete_label(&mut self, name: &str) -> Result<(), String> {
        if self.labels.is_none() {
            return Err("Task has no labels".to_string());
        }

        let index = self.labels.as_ref().unwrap().iter().position(|label| label.name == name);

        if index.is_none() {
            return Err(format!("Label with name '{name}' not found"));
        }

        self.labels.as_mut().unwrap().remove(index.unwrap());

        Ok(())
    }

    pub fn get_label_by_name(&self, name: &str) -> Option<&Label> {
        self.labels
            .as_ref()
            .and_then(|labels| labels.iter().find(|label| label.name == name))
    }
}

impl Comment {
    pub fn new(id: String, props: HashMap<String, String>, text: String) -> Comment {
        Comment {
            id: Some(id),
            props,
            text,
        }
    }

    pub fn get_id(&self) -> Option<String> {
        match &self.id {
            Some(id) => Some(id.clone()),
            _ => None
        }
    }

    pub fn set_id(&mut self, id: String) {
        self.id = Some(id);
    }

    pub fn get_all_properties(&self) -> &HashMap<String, String> {
        &self.props
    }

    pub fn get_text(&self) -> String {
        self.text.to_string()
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }
}

impl Label {
    pub fn new(name: String, color: Option<String>, description: Option<String>) -> Label {
        Label {
            name, color, description
        }
    }

    pub fn get_name(&self) -> String {
        self.name.to_string()
    }

    pub fn get_color(&self) -> String {
        self.color.clone().unwrap_or_else(|| String::from(""))
    }

    pub fn set_color(&mut self, color: String) {
        self.color = Some(color);
    }

    pub fn get_description(&self) -> Option<String> {
        self.description.clone()
    }

    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }
}

macro_rules! map_err {
    ($expr:expr) => {
        $expr.map_err(|e| e.message().to_owned())?
    }
}


impl TaskContext {
    pub fn new(repository_path: String) -> Self {
        Self {
            repository_path,
        }
    }

    pub fn list_tasks(&self) -> Result<Vec<Task>, String> {
        let repo = map_err!(Repository::discover(&self.repository_path));
        let task_ref = map_err!(repo.find_reference(&self.get_ref_path()));
        let task_tree = map_err!(task_ref.peel_to_tree());

        let mut result = vec![];

        let _ = map_err!(task_tree.walk(TreeWalkMode::PreOrder, |_, entry| {
            let oid = entry.id();
            let blob = repo.find_blob(oid).unwrap();
            let content = blob.content();

            let task = serde_json::from_slice(content).unwrap();
            result.push(task);

            TreeWalkResult::Ok
        }));

        Ok(result)
    }

    pub fn find_task(&self, id: &str) -> Result<Option<Task>, String> {
        let repo = map_err!(Repository::discover(&self.repository_path));
        let task_ref = repo.find_reference(&self.get_ref_path());
        match task_ref {
            Ok(task_ref) => {
                let task_tree = map_err!(task_ref.peel_to_tree());
                let result = match task_tree.get_name(id) {
                    Some(entry) => {
                        let oid = entry.id();
                        let blob = map_err!(repo.find_blob(oid));
                        let content = blob.content();
                        let task = serde_json::from_slice(content).unwrap();

                        Some(task)
                    },
                    None => None,
                };

                Ok(result)
            },
            Err(_) => Ok(None)
        }
    }

    fn get_actions_from_history(
        &self,
        task_id: &str,
        repo: &Repository,
        commit: Commit,
        limit: u16) -> Result<Vec<Option<TaskAction>>, String> {
        let mut counter = 0;
        let mut current_commit = commit;
        let mut actions: Vec<Option<TaskAction>> = vec![];
        while counter < limit {
            let tree = map_err!(current_commit.tree());
            match tree.get_name(format!("action-{}", task_id).as_str()) {
                None => {
                    // TODO?
                    actions.push(None);
                },
                Some(entry) => {
                    let oid = entry.id();
                    let blob = map_err!(repo.find_blob(oid));
                    let content = blob.content();
                    let action = serde_json::from_slice(content).unwrap();
                    actions.push(Some(action));
                }
            }
            if current_commit.parent_count() <= 0 {
                break;
            }
            // TODO, this only allows for a linear parent tree
            counter += 1;
            current_commit = map_err!(current_commit.parent(0));
        }

        actions.reverse();
        Ok(actions)
    }
    pub fn get_task_history(&self, id: &str) -> Result<Vec<Option<TaskAction>>, String> {
        let repo = map_err!(Repository::discover(&self.repository_path));
        let task_ref = &repo.find_reference(&self.get_ref_path());
        match task_ref {
            Ok(task_ref) => {
                let commit = map_err!(task_ref.peel_to_commit());
                self.get_actions_from_history(id, &repo, commit, 10)
                // let task_tree = map_err!(task_ref.peel_to_tree());
                // let commit = task_ref.peel_to_commit().unwrap();
                // let parents = commit.parents();
                // let action_id = format!("action-{}", id);
                // let mut actions: Vec<Option<TaskAction>> = parents.map(|p| {
                //     let tree = p.tree().unwrap();
                //     match tree.get_name(action_id.as_str()) {
                //         None => None,
                //         Some(entry) => {
                //             let oid = entry.id();
                //             let blob = repo.find_blob(oid).unwrap();
                //             let content = blob.content();
                //             let task = serde_json::from_slice(content).unwrap();
                //             // task.action
                //             Some(task)
                //         }
                //     }
                // }).collect();
                // let latest_action = match task_tree.get_name(action_id.as_str()) {
                //     Some(entry) => {
                //         let oid = entry.id();
                //         let blob = map_err!(repo.find_blob(oid));
                //         let content = blob.content();
                //         let task  = serde_json::from_slice(content).unwrap();
                //         Some(task)
                //     },
                //     None => None,
                // };
                // actions.push(latest_action);
                // Ok(actions)
            }
            Err(e) => Err(e.message().to_owned())
        }
    }

    pub fn delete_tasks(&self, ids: &[&str]) -> Result<(), String> {
        let repo = map_err!(Repository::discover(&self.repository_path));
        let task_ref = map_err!(repo.find_reference(&self.get_ref_path()));
        let task_tree = map_err!(task_ref.peel_to_tree());

        let mut treebuilder = map_err!(repo.treebuilder(Some(&task_tree)));
        for id in ids {
            map_err!(treebuilder.remove(id));
        }
        let tree_oid = map_err!(treebuilder.write());

        let parent_commit = map_err!(task_ref.peel_to_commit());
        let parents = vec![parent_commit];
        let me = &map_err!(repo.signature());

        let mut ids = ids.iter().map(|id| id.parse::<u64>().unwrap()).collect::<Vec<_>>();
        ids.sort();
        let ids = ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(", ");
        map_err!(repo.commit(Some(&self.get_ref_path()), me, me, format!("Delete task {}", ids).as_str(), &map_err!(repo.find_tree(tree_oid)), &parents.iter().collect::<Vec<_>>()));

        Ok(())
    }
    pub fn clear_tasks(&self) -> Result<u64, String> {
        let repo = map_err!(Repository::discover(&self.repository_path));
        let task_ref = map_err!(repo.find_reference(&self.get_ref_path()));
        let task_tree = map_err!(task_ref.peel_to_tree());

        let mut treebuilder = map_err!(repo.treebuilder(Some(&task_tree)));
        // There will be 2x the number of tasks, since an "Action" blob will appear next to the task.
        let task_count = (treebuilder.len() / 2) as u64;
        map_err!(treebuilder.clear());
        let tree_oid = map_err!(treebuilder.write());

        let parent_commit = map_err!(task_ref.peel_to_commit());
        let parents = vec![parent_commit];
        let me = &map_err!(repo.signature());

        map_err!(repo.commit(Some(&self.get_ref_path()), me, me, "Clear tasks", &map_err!(repo.find_tree(tree_oid)), &parents.iter().collect::<Vec<_>>()));

        Ok(task_count)
    }

    pub fn create_task(&self, mut task: Task) -> Result<Task, String> {
        let repo = map_err!(Repository::discover(&self.repository_path));
        let task_ref_result = repo.find_reference(&self.get_ref_path());
        let source_tree = match task_ref_result {
            Ok(ref reference) => {
                match reference.peel_to_tree() {
                    Ok(tree) => Some(tree),
                    _ => None
                }
            }
            _ => { None }
        };

        if task.get_id().is_none() {
            let id = self.get_next_id().unwrap_or_else(|_| "1".to_string());
            task.set_id(id);
        }
        let string_content = serde_json::to_string(&task).unwrap();
        let content = string_content.as_bytes();
        let oid = map_err!(repo.blob(content));
        let string_action = serde_json::to_string(&TaskAction::TaskCreate).unwrap(); // TODO
        let action_content = string_action.as_bytes();
        let action_oid = map_err!(repo.blob(action_content));
        let mut treebuilder = map_err!(repo.treebuilder(source_tree.as_ref()));
        map_err!(treebuilder.insert(&task.get_id().unwrap(), oid, FileMode::Blob.into()));
        let action_name= format!("action-{}", &task.get_id().unwrap());
        map_err!(treebuilder.insert(action_name, action_oid, FileMode::Blob.into()));
        let tree_oid = map_err!(treebuilder.write());

        let me = &map_err!(repo.signature());
        let mut parents = vec![];
        if task_ref_result.is_ok() {
            let parent_commit = map_err!(task_ref_result).peel_to_commit();
            if parent_commit.is_ok() {
                parents.push(map_err!(parent_commit));
            }
        }
        map_err!(repo.commit(Some(&self.get_ref_path()), me, me, format!("Create task {}", &task.get_id().unwrap_or_else(|| String::from("?"))).as_str(), &map_err!(repo.find_tree(tree_oid)), &parents.iter().collect::<Vec<_>>()));

        Ok(task)
    }

    pub fn update_task_v2(&self, task: Task, action: Option<TaskAction>) -> Result<String, String> {
        let repo = map_err!(Repository::discover(&self.repository_path));
        let task_ref_result = map_err!(repo.find_reference(&self.get_ref_path()));
        let parent_commit = map_err!(task_ref_result.peel_to_commit());
        let source_tree = map_err!(task_ref_result.peel_to_tree());
        let string_content = serde_json::to_string(&task).unwrap();
        let content = string_content.as_bytes();
        let oid = map_err!(repo.blob(content));
        let mut treebuilder = map_err!(repo.treebuilder(Some(&source_tree)));
        map_err!(treebuilder.insert(&task.get_id().unwrap(), oid, FileMode::Blob.into()));
        if let Some(action) = action {
            let string_action = serde_json::to_string(&action).unwrap(); // TODO
            let action_content = string_action.as_bytes();
            let action_oid = map_err!(repo.blob(action_content));
            let action_name= format!("action-{}", &task.get_id().unwrap());
            map_err!(treebuilder.insert(action_name, action_oid, FileMode::Blob.into()));
        }
        let tree_oid = map_err!(treebuilder.write());

        let me = &map_err!(repo.signature());
        let parents = vec![parent_commit];
        map_err!(repo.commit(Some(&self.get_ref_path()), me, me, format!("Update task {}", &task.get_id().unwrap()).as_str(), &map_err!(repo.find_tree(tree_oid)), &parents.iter().collect::<Vec<_>>()));

        Ok(task.get_id().unwrap())
    }
    pub fn update_task(&self, task: Task) -> Result<String, String> {
        self.update_task_v2(task, None)
        // let repo = map_err!(Repository::discover(&self.repository_path));
        // let task_ref_result = map_err!(repo.find_reference(&self.get_ref_path()));
        // let parent_commit = map_err!(task_ref_result.peel_to_commit());
        // let source_tree = map_err!(task_ref_result.peel_to_tree());
        // let string_content = serde_json::to_string(&task).unwrap();
        // let content = string_content.as_bytes();
        // let oid = map_err!(repo.blob(content));
        // // let action = map_err!(repo.blob("UPDATE".as_bytes()));
        // let mut treebuilder = map_err!(repo.treebuilder(Some(&source_tree)));
        // map_err!(treebuilder.insert(&task.get_id().unwrap(), oid, FileMode::Blob.into()));
        // let tree_oid = map_err!(treebuilder.write());
        //
        // let me = &map_err!(repo.signature());
        // let parents = vec![parent_commit];
        // map_err!(repo.commit(Some(&self.get_ref_path()), me, me, format!("Update task {}", &task.get_id().unwrap()).as_str(), &map_err!(repo.find_tree(tree_oid)), &parents.iter().collect::<Vec<_>>()));
        //
        // Ok(task.get_id().unwrap())
    }

    fn get_next_id(&self) -> Result<String, String> {
        let repo = map_err!(Repository::discover(&self.repository_path));
        let task_ref = map_err!(repo.find_reference(&self.get_ref_path()));
        let task_tree = map_err!(task_ref.peel_to_tree());

        let mut result = 0;

        let _ = map_err!(task_tree.walk(TreeWalkMode::PreOrder, |_, entry| {
            let entry_name = entry.name().unwrap();
            match entry_name.parse::<i64>() {
                Ok(id) => {
                    if id > result {
                        result = id;
                    }
                },
                _ => return TreeWalkResult::Skip
            };

            TreeWalkResult::Ok
        }));

        Ok((result + 1).to_string())
    }

    pub fn update_task_id(&self, id: &str, new_id: &str) -> Result<(), String> {
        let mut task = self.find_task(&id)?.unwrap();
        task.set_id(new_id.to_string());
        self.create_task(task)?;
        self.delete_tasks(&[&id])?;

        Ok(())
    }

    pub fn update_comment_id(&self, task_id: &str, id: &str, new_id: &str) -> Result<(), String> {
        let mut task = self.find_task(&task_id)?.unwrap().clone();
        let comments = task.get_comments();
        match comments {
            Some(comments) => {
                let updated_comments = comments.iter().map(|c| {
                    if c.get_id().unwrap() == id {
                        let mut c = c.clone();
                        c.set_id(new_id.to_string());
                        c
                    } else {
                        c.clone()
                    }
                }).collect::<Vec<_>>();
                task.set_comments(updated_comments);
                self.update_task(task)?;
            },
            None => {}
        }

        Ok(())
    }

    pub fn list_remotes(&self, remote: &Option<String>) -> Result<Vec<String>, String> {
        let repo = map_err!(Repository::discover(&self.repository_path));
        let remotes = map_err!(repo.remotes());
        Ok(remotes.iter()
            .filter(|s| remote.is_none() || remote.as_ref().unwrap().as_str() == s.unwrap())
            .map(|s| repo.find_remote(s.unwrap()).unwrap().url().unwrap().to_owned())
            .collect())
    }
    pub fn get_config_value(&self, key: &str) -> Result<String, String> {
        let repo = map_err!(Repository::discover(&self.repository_path));
        let config = map_err!(repo.config());
        Ok(map_err!(config.get_string(key)))
    }
    pub fn get_current_user(&self) -> Result<Option<String>, String> {
        let repo = map_err!(Repository::discover(&self.repository_path));
        let me = &map_err!(repo.signature());
        match me.name() {
            Some(name) => Ok(Some(String::from(name))),
            _ => match me.email() {
                Some(email) => Ok(Some(String::from(email))),
                _ => Ok(None),
            }
        }
    }
    pub fn get_ref_path(&self) -> String {
        self.get_config_value("task.ref").unwrap_or_else(|_| "refs/tasks/tasks".to_string())
    }
    pub fn set_config_value(&self, key: &str, value: &str) -> Result<(), String> {
        let repo = map_err!(Repository::discover(&self.repository_path));
        let mut config = map_err!(repo.config());
        map_err!(config.set_str(key, value));
        Ok(())
    }

    pub fn set_ref_path(&self, ref_path: &str, move_ref: bool) -> Result<(), String> {
        let repo = map_err!(Repository::discover(&self.repository_path));

        let current_reference = repo.find_reference(&self.get_ref_path());
        if let Ok(current_reference) = &current_reference {
            let commit = map_err!(current_reference.peel_to_commit());
            map_err!(repo.reference(ref_path, commit.id(), true, "task.ref migrated"));
        }

        let mut config = map_err!(repo.config());
        map_err!(config.set_str("task.ref", ref_path));

        if move_ref && current_reference.is_ok() {
            map_err!(current_reference.unwrap().delete());
        }

        Ok(())
    }
}
fn get_current_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

#[cfg(test)]
mod test {
    use crate::*;
    use git2::Repository;
    use std::collections::HashMap;
    use std::env::temp_dir;
    use uuid::Uuid;

    #[test]
    fn test_ref_path() {
        let repo_dir = temp_dir().join(Uuid::new_v4().to_string());
        std::fs::create_dir_all(repo_dir.clone()).unwrap();
        let repo = Repository::init(repo_dir.clone()).unwrap();
        let context = TaskContext::new(repo_dir.display().to_string());

        let ref_path = context.get_ref_path();
        assert!(context.set_ref_path("refs/heads/test-git-task", true).is_ok());
        assert_eq!(context.get_ref_path(), "refs/heads/test-git-task");
        assert!(context.set_ref_path(&ref_path, true).is_ok());
        assert_eq!(context.get_ref_path(), ref_path);

        assert!(repo.is_empty().unwrap());

        std::fs::remove_dir_all(repo_dir).unwrap();
    }

    #[test]
    fn test_create_update_delete_task() {
        let repo_dir = temp_dir().join(Uuid::new_v4().to_string());
        std::fs::create_dir_all(repo_dir.clone()).unwrap();
        let _repo = Repository::init(repo_dir.clone()).unwrap();
        let context = TaskContext::new(repo_dir.display().to_string());

        let id = context.get_next_id().unwrap_or_else(|_| "1".to_string());
        let task = Task::construct_task(
            "Test task".to_string(),
            "Description goes here".to_string(),
            "OPEN".to_string(),
            context.get_current_user().unwrap(),
            Some(get_current_timestamp()));
        let create_result = context.create_task(task);
        assert!(create_result.is_ok());
        let mut task = create_result.unwrap();
        assert_eq!(task.get_id(), Some(id.clone()));
        assert_eq!(task.get_property("name").unwrap(), "Test task");
        assert_eq!(task.get_property("description").unwrap(), "Description goes here");
        assert_eq!(task.get_property("status").unwrap(), "OPEN");
        assert!(task.has_property("created"));

        task.set_property("description", "Updated description");
        let comment_props = HashMap::from([("author".to_string(), "Some developer".to_string())]);
        task.add_comment(None, comment_props, "This is a comment".to_string(), context.get_current_user().unwrap());
        task.set_property("custom_prop", "Custom content");
        let update_result = context.update_task(task);
        assert!(update_result.is_ok());
        assert_eq!(update_result.unwrap(), id.clone());

        let find_result = context.find_task(&id);
        assert!(find_result.is_ok());
        let task = find_result.unwrap();
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.get_id(), Some(id.clone()));
        assert_eq!(task.get_property("description").unwrap(), "Updated description");
        let comments = task.get_comments().clone();
        assert!(comments.is_some());
        let comments = comments.unwrap();
        assert_eq!(comments.len(), 1);
        let comment = comments.first().unwrap();
        assert_eq!(comment.get_text(), "This is a comment".to_string());
        let comment_props = comment.clone().props;
        assert_eq!(comment_props.get("author").unwrap(), &"Some developer".to_string());
        assert_eq!(task.get_property("custom_prop").unwrap(), "Custom content");

        let delete_result = context.delete_tasks(&[&id]);
        assert!(delete_result.is_ok());

        let find_result = context.find_task(&id);
        assert!(find_result.is_ok());
        let task = find_result.unwrap();
        assert!(task.is_none());

        std::fs::remove_dir_all(repo_dir).unwrap();
    }

    #[test]
    fn test_update_comment_id() {
        let repo_dir = temp_dir().join(Uuid::new_v4().to_string());
        std::fs::create_dir_all(repo_dir.clone()).unwrap();
        let _repo = Repository::init(repo_dir.clone()).unwrap();
        let context = TaskContext::new(repo_dir.display().to_string());

        // Create a task first
        let id = context.get_next_id().unwrap_or_else(|_| "1".to_string());
        let task = Task::construct_task(
            "Test task".to_string(),
            "Description goes here".to_string(),
            "OPEN".to_string(),
            context.get_current_user().unwrap(),
            Some(get_current_timestamp())
        );
        let create_result = context.create_task(task);
        assert!(create_result.is_ok());
        let mut task = create_result.unwrap();

        // Add a comment to the task
        let comment_props = HashMap::from([("author".to_string(), "Some developer".to_string())]);
        let comment = task.add_comment(
            Some("1".to_string()),
            comment_props,
            "Test comment".to_string(),
            context.get_current_user().unwrap(),
        );
        assert_eq!(comment.get_id().unwrap(), "1");
        let update_result = context.update_task(task);
        assert!(update_result.is_ok());

        // Update the comment ID
        let result = context.update_comment_id(&id, "1", "2");
        assert!(result.is_ok());

        // Verify the comment ID was updated
        let updated_task = context.find_task(&id).unwrap().unwrap();
        let updated_comments = updated_task.get_comments().as_ref().unwrap();
        assert_eq!(updated_comments.len(), 1);
        assert_eq!(updated_comments[0].get_id().unwrap(), "2");

        // Clean up
        let delete_result = context.delete_tasks(&[&id]);
        assert!(delete_result.is_ok());

        std::fs::remove_dir_all(repo_dir).unwrap();
    }

    #[test]
    fn test_get_task_history() {
        let repo_dir = temp_dir().join(Uuid::new_v4().to_string());
        std::fs::create_dir_all(repo_dir.clone()).unwrap();
        let _repo = Repository::init(repo_dir.clone()).unwrap();
        let context = TaskContext::new(repo_dir.display().to_string());

        // Task Create
        let task = Task::construct_task(
            "Test task".to_string(),
            "Description goes here".to_string(),
            "OPEN".to_string(),
            context.get_current_user().unwrap(),
            Some(get_current_timestamp())
        );
        let mut task = context.create_task(task).unwrap();

        // Update task status
        task.set_property("status", "IN_PROGRESS");
        context.update_task_v2(task.clone(), Some(TaskAction::UpdateStatus)).unwrap();
        // Set a property
        // Edit a property
        // Delete a property
        // Search & replace property values
        // Add a comment
        let comment_props = HashMap::from([("author".to_string(), "Some developer".to_string())]);
        let _ = task.add_comment(
            Some("1".to_string()),
            comment_props,
            "Test comment".to_string(),
            context.get_current_user().unwrap(),
        );
        let task_id = context.update_task_v2(task, Some(TaskAction::AddComment)).unwrap();
        // Delete a comment
        // Add a label
        // Update a label
        // Delete a label
        // Out of scope:
        // 1. Pushing/pulling from remotes
        // 2. Deleting tasks entirely (do this someday)

        let task_history = context.get_task_history(&task_id);
        assert!(task_history.is_ok());
        let mut task_history = task_history.unwrap();
        assert_eq!((&task_history).len(), 3);
        let expected_task_history: Vec<Option<TaskAction>> = vec!(
            Some(TaskAction::TaskCreate),
            Some(TaskAction::UpdateStatus),
            Some(TaskAction::AddComment),
        );
        assert_eq!(task_history, expected_task_history);

        let latest = task_history.pop().unwrap();
        assert_eq!(latest, Some(TaskAction::AddComment));

        std::fs::remove_dir_all(repo_dir).unwrap();
    }

    #[test]
    fn test_clear_tasks() {
        let repo_dir = temp_dir().join(Uuid::new_v4().to_string());
        std::fs::create_dir_all(repo_dir.clone()).unwrap();
        let _repo = Repository::init(repo_dir.clone()).unwrap();
        let context = TaskContext::new(repo_dir.display().to_string());

        let id = context.get_next_id().unwrap_or_else(|_| "1".to_string());
        let task = Task::construct_task(
            "Test task".to_string(),
            "Description goes here".to_string(),
            "OPEN".to_string(),
            context.get_current_user().unwrap(),
            Some(get_current_timestamp()));
        let create_result = context.create_task(task);
        assert!(create_result.is_ok());
        let task = create_result.unwrap();
        assert_eq!(task.get_id(), Some(id.clone()));

        let id = context.get_next_id().unwrap_or_else(|_| "2".to_string());
        let task2 = Task::construct_task(
            "Another task".to_string(),
            "Another description".to_string(),
            "IN_PROGRESS".to_string(),
            context.get_current_user().unwrap(),
            Some(get_current_timestamp())
        );
        let create_result2 = context.create_task(task2);
        assert!(create_result2.is_ok());
        let task2 = create_result2.unwrap();
        assert_eq!(task2.get_id(), Some(id.clone()));

        let id = context.get_next_id().unwrap_or_else(|_| "3".to_string());
        let task3 = Task::construct_task(
            "Third task".to_string(),
            "Third description".to_string(),
            "CLOSED".to_string(),
            context.get_current_user().unwrap(),
            Some(get_current_timestamp()));
        let create_result3 = context.create_task(task3);
        assert!(create_result3.is_ok());
        let task3 = create_result3.unwrap();
        assert_eq!(task3.get_id(), Some(id.clone()));

        let clear_result = context.clear_tasks();
        assert!(clear_result.is_ok());
        assert_eq!(clear_result.unwrap(), 3);

        let find_result = context.find_task(&id);
        assert!(find_result.is_ok());
        let task = find_result.unwrap();
        assert!(task.is_none());

        let find_result = context.find_task(&task2.get_id().unwrap());
        assert!(find_result.is_ok());
        let task = find_result.unwrap();
        assert!(task.is_none());

        let find_result = context.find_task(&task3.get_id().unwrap());
        assert!(find_result.is_ok());
        let task = find_result.unwrap();
        assert!(task.is_none());

        std::fs::remove_dir_all(repo_dir).unwrap();
    }
}