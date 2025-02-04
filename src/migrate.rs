use std::{
    fs::{self},
    io::ErrorKind,
    path::PathBuf,
};

use crate::{utils::path_from_iter, Config, OldConfig, OldProject, FOLDER_PREFIX, PROJECTS_FOLDER};

pub fn migrate_to_new_config(old: OldConfig) -> Config {
    println!("Migrating to new Wechsel setup");

    let new_base_folder = path_from_iter([
        dirs::home_dir().expect("Could not find home directory"),
        PathBuf::from(&old.all_prjs.name),
    ]);
    let old_root_folder = PathBuf::from(&old.all_prjs.path);

    if !new_base_folder.exists() || !new_base_folder.is_dir() {
        if let Err(e) = std::os::unix::fs::symlink(&old_root_folder, &new_base_folder) {
            if matches!(e.kind(), ErrorKind::AlreadyExists) {
                eprintln!("Could not create symlink of old root project {e:?}")
            }
        }
    }

    fn recurse(prj: &OldProject, path: PathBuf) {
        let new_path = path_from_iter([path.clone(), PathBuf::from(PROJECTS_FOLDER)]);

        if !new_path.exists() || !new_path.is_dir() {
            fs::create_dir(&new_path)
                .unwrap_or_else(|e| panic!("Could not create projects folder: {new_path:?} {}", e));
        }

        for child in &prj.children {
            let child_path = path_from_iter([path.clone(), PathBuf::from(&child.path)]);
            let new_child_path = path_from_iter([new_path.clone(), PathBuf::from(&child.name)]);

            if let Err(e) = std::os::unix::fs::symlink(&child_path, &new_child_path) {
                if !matches!(e.kind(), ErrorKind::AlreadyExists) {
                    eprintln!("Could not create symlink of old project {child_path:?}")
                }
            }

            recurse(child, child_path.clone())
        }

        // Rename folders to {name}.wechsel
        for folder in prj.folder.iter() {
            if folder.starts_with(FOLDER_PREFIX) {
                continue;
            }
            let mut folder_path = PathBuf::from(&path);
            let new_folder_path = path_from_iter([
                folder_path.clone(),
                PathBuf::from(format!("{FOLDER_PREFIX}{folder}")),
            ]);

            if !new_folder_path.exists() {
                folder_path.push(folder);
                if let Err(e) = fs::rename(&folder_path, &new_folder_path) {
                    if !matches!(e.kind(), ErrorKind::NotFound) {
                        eprintln!(
                            "Renaming projects {} folder {folder_path:?} to {new_folder_path:?} failed",
                            &prj.name
                        )
                    }
                }
            }
            if new_folder_path.exists() && PathBuf::from(folder).components().nth(1).is_some() {
                // The folders are not directly in the project folder
                let target = path_from_iter([
                    PathBuf::from(&path),
                    PathBuf::from(new_folder_path.file_name().unwrap()),
                ]);

                if let Err(e) = std::os::unix::fs::symlink(&new_folder_path, &target) {
                    if !matches!(e.kind(), ErrorKind::AlreadyExists) {
                        eprintln!("Could not create symlink of project folder {new_folder_path:?}")
                    }
                }
            }
        }
    }

    recurse(&old.all_prjs, new_base_folder.clone());

    Config {
        active: old.active,
        base_folder: new_base_folder,
    }
}
