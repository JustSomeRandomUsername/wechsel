use std::{fs, path::PathBuf};

use serde::Serialize;

use crate::{utils::path_from_iter, PROJECTS_FOLDER};

#[derive(Serialize)]
pub struct ProjectTreeNode {
    name: String,
    children: Vec<ProjectTreeNode>,
}

pub fn get_project_tree(project_path: PathBuf) -> Option<ProjectTreeNode> {
    let projects = path_from_iter([&project_path, &PathBuf::from(PROJECTS_FOLDER)]);

    let children = fs::read_dir(&projects)
        .map(|children| {
            children
                .into_iter()
                .filter_map(|child| {
                    let Ok(child) = child else { return None };
                    let child_path = child.path();
                    get_project_tree(child_path.clone())
                })
                .collect()
        })
        .unwrap_or_default();

    Some(ProjectTreeNode {
        name: project_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string(),
        children,
    })
}
