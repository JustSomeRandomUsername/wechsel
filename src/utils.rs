use std::{
    fs::{self, DirEntry},
    io::Error,
    path::{Path, PathBuf},
};

use crate::WECHSEL_FOLDER_EXTENSION;

pub const HOME_FOLDERS: [&str; 6] = [
    "Desktop",
    "Downloads",
    "Documents",
    "Pictures",
    "Videos",
    "Music",
];

// /**
//  * Searches for the project with the target name
//  *
//  * returns a None if target is not found and a Vector with the path of the target project at position 0 followed by the paths of its parents e.g. [path, parent_prj_path, grandparent_prj_path]
//  */
// pub fn search_for_project(target_prj_name: &str, tree: &ProjectTreeNode) -> Option<Vec<PathBuf>> {
//     pub fn inner_search_for_project(
//         project_path: PathBuf,
//         target: &str,
//         tree: &ProjectTreeNode,
//     ) -> Option<Vec<PathBuf>> {
//         if project_path
//             .file_stem()
//             .map(|stem| stem == target)
//             .unwrap_or(false)
//         {
//             return Some(vec![project_path]);
//         }

//         for child in tree.children.iter() {
//             let child_path = path_from_iter([
//                 &project_path,
//                 &PathBuf::from(format!("{}.{PROJECT_EXTENSION}", child.prj_name)),
//             ]);
//             if let Some(mut result) = inner_search_for_project(child_path, target, child) {
//                 result.push(project_path);
//                 return Some(result);
//             }
//         }
//         None
//     }

//     inner_search_for_project(
//         dirs::home_dir().expect("Could not find home dir"),
//         target_prj_name,
//         tree,
//     )
// }

//** Find subfolders of target path that have the wechsel extension*/
pub fn get_folders(path: &PathBuf) -> Vec<PathBuf> {
    fs::read_dir(path)
        .ok()
        .map(|children| {
            children
                .into_iter()
                .filter_map(|file| {
                    is_entry_folder_with_extension(&file, WECHSEL_FOLDER_EXTENSION)
                        .map(|file| file.path())
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn path_from_iter<const N: usize, S: AsRef<Path>>(inp: [S; N]) -> PathBuf {
    inp.into_iter().collect()
}

pub fn is_entry_folder_with_extension<'a>(
    entry: &'a Result<DirEntry, Error>,
    extension: &str,
) -> Option<&'a DirEntry> {
    entry.as_ref().ok().and_then(|entry| {
        (entry
            .file_type()
            .map(|file_type| file_type.is_dir())
            .unwrap_or(false)
            && entry
                .path()
                .extension()
                .map(|ext| ext == extension)
                .unwrap_or(false))
        .then_some(entry)
    })
}

pub fn query_active_project() -> Option<String> {
    let project_folder_path = path_from_iter([
        dirs::home_dir().expect("Could not find home directory"),
        PathBuf::from("Project"),
    ]);

    if project_folder_path.is_symlink() {
        if let Ok(target) = fs::read_link(project_folder_path) {
            return target
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(|a| a.to_string());
        }
    }
    None
}
