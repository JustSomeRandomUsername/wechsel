use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{FOLDER_PREFIX, PROJECTS_FOLDER};

pub const HOME_FOLDERS: [&str; 6] = [
    "Desktop",
    "Downloads",
    "Documents",
    "Pictures",
    "Videos",
    "Music",
];
pub fn search_for_project(mut project_path: PathBuf, target: &str) -> Option<Vec<PathBuf>> {
    if project_path
        .file_name()
        .map(|name| name == target)
        .unwrap_or(false)
    {
        return Some(vec![project_path]);
    }

    project_path.push(PROJECTS_FOLDER);

    let Ok(children) = fs::read_dir(&project_path) else {
        return None;
    };

    for child in children.into_iter() {
        let Ok(child) = child else {
            continue;
        };
        let child_path = child.path();
        if let Some(mut result) = search_for_project(child_path.clone(), target) {
            project_path.pop();
            result.push(project_path);
            return Some(result);
        }
    }

    None
}

//** Find subfolders of target path that have the wechsel extension*/
pub fn get_folders(path: &PathBuf) -> Vec<PathBuf> {
    fs::read_dir(path)
        .ok()
        .map(|children| {
            children
                .into_iter()
                .filter_map(|file| {
                    file.map(|file| file.path()).ok().and_then(|path| {
                        path.file_name()
                            .and_then(|name| name.to_str())
                            .map(|name| name.starts_with(FOLDER_PREFIX))
                            .unwrap_or(false)
                            .then_some(path)
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn path_from_iter<const N: usize, S: AsRef<Path>>(inp: [S; N]) -> PathBuf {
    inp.into_iter().collect()
}
