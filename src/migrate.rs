use std::{
    fs::{self},
    io::ErrorKind,
    path::PathBuf,
};

use serde::Deserialize;

use crate::{utils::path_from_iter, PROJECT_EXTENSION, WECHSEL_FOLDER_EXTENSION};

const OLD_CONFIG_NAME: &str = "wechsel_projects.json";

#[derive(Deserialize)]
struct OldConfig {
    active: String,
    all_prjs: OldProject,
}

#[derive(Deserialize)]
struct OldProject {
    name: String,
    path: String,
    folder: Vec<String>,
    children: Vec<OldProject>,
}
pub fn migrate_to_new_config(config_dir: &PathBuf) {
    let config_path: PathBuf = path_from_iter([config_dir.clone(), PathBuf::from(OLD_CONFIG_NAME)]);

    // Load Config
    if !config_path.exists() {
        panic!("No config file, it looks like your wechsel setup is was never initialized or is already the new kind")
    }
    let contents =
        fs::read_to_string(&config_path).expect("Should have been able to read the file");

    if let Ok(conf) = serde_json::from_str::<OldConfig>(&contents) {
        perform_migration(conf);
    }

    let new_config_path = config_path.with_extension(".json.old");
    if let Err(e) = fs::rename(&config_path, &new_config_path) {
        if !matches!(e.kind(), ErrorKind::NotFound) {
            eprintln!("Renaming config Path from {config_path:?} to {new_config_path:?} failed",)
        }
    }
}
fn perform_migration(old: OldConfig) {
    println!("Migrating to new Wechsel setup");

    fn recurse(prj: &OldProject, path: PathBuf) {
        let prj_folder = path_from_iter([
            &path,
            &PathBuf::from(format!("{}.{}", prj.name, PROJECT_EXTENSION)),
        ]);

        if !prj_folder.exists() || !prj_folder.is_dir() {
            fs::create_dir(&prj_folder).unwrap_or_else(|e| {
                panic!("Could not create project folder: {prj_folder:?} {}", e)
            });
        }

        for child in &prj.children {
            let child_path = path_from_iter([path.clone(), PathBuf::from(&child.path)]);
            let new_child_path = path_from_iter([
                &prj_folder,
                &PathBuf::from(format!("{}.{}", child.name, WECHSEL_FOLDER_EXTENSION)),
            ]);

            if let Err(e) = std::os::unix::fs::symlink(&child_path, &new_child_path) {
                if !matches!(e.kind(), ErrorKind::AlreadyExists) {
                    eprintln!("Could not create symlink of old project {child_path:?}")
                }
            }

            recurse(child, child_path.clone())
        }

        // Rename folders wechsel folders
        for folder in prj.folder.iter() {
            if folder.ends_with(&format!(".{WECHSEL_FOLDER_EXTENSION}")) {
                continue; // It already does
            }
            let folder_path = path_from_iter([&path, &PathBuf::from(folder)]);

            let new_folder_path = PathBuf::from(format!("{folder}.{WECHSEL_FOLDER_EXTENSION}"));
            let directly_in_project = new_folder_path.components().nth(1).is_some();
            let new_folder_path =
                path_from_iter([&path, &PathBuf::from(new_folder_path.file_name().unwrap())]);

            if !new_folder_path.exists() {
                if !directly_in_project {
                    // rename the folder
                    if let Err(e) = fs::rename(&folder_path, &new_folder_path) {
                        if !matches!(e.kind(), ErrorKind::NotFound) {
                            eprintln!(
                                "Renaming projects {} folder {folder_path:?} to {new_folder_path:?} failed",
                                &prj.name
                            )
                        }
                    }
                } else {
                    // folder symlink it the folders that are not directly in the project folder
                    if let Err(e) = std::os::unix::fs::symlink(&folder_path, &new_folder_path) {
                        if !matches!(e.kind(), ErrorKind::AlreadyExists) {
                            eprintln!(
                                "Could not create symlink of project folder {new_folder_path:?}"
                            )
                        }
                    }
                }
            }
        }
    }

    recurse(
        &old.all_prjs,
        dirs::home_dir().expect("Could not find home directory"),
    );
}
