use std::{
    fs::{self},
    io::ErrorKind,
    path::{Path, PathBuf},
};

use serde::Deserialize;
#[cfg(test)]
use serde::Serialize;

use crate::{PROJECT_EXTENSION, WECHSEL_FOLDER_EXTENSION, utils::path_from_iter};

const OLD_CONFIG_NAME: &str = "wechsel_projects.json";

#[derive(Deserialize)]
#[cfg_attr(test, derive(Serialize))]
pub struct OldConfig {
    pub active: String,
    pub all_prjs: OldProject,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub struct OldProject {
    pub name: String,
    pub path: String,
    pub folder: Vec<String>,
    pub children: Vec<OldProject>,
}

pub fn get_old_config_file_path_unchecked(config_dir: &Path) -> PathBuf {
    path_from_iter([config_dir, PathBuf::from(OLD_CONFIG_NAME).as_path()])
}

pub fn get_old_config_file_path(config_dir: &Path) -> Option<PathBuf> {
    let path = get_old_config_file_path_unchecked(config_dir);
    path.exists().then_some(path)
}
pub fn migrate_to_new_config(config_dir: &Path, no_prompts: bool) {
    let Some(config_path) = get_old_config_file_path(config_dir) else {
        eprintln!("No config file, it looks like your wechsel setup has no need to be migrated");
        std::process::exit(1);
    };

    let contents =
        fs::read_to_string(&config_path).expect("Should have been able to read the file");

    if !no_prompts {
        println!("You are about to start the migration to the new wechsel setup.");
        println!(
            "To do this this script will move your old projects into a new folder tree in your home directory"
        );
        match inquire::prompt_confirmation("Are you sure you want this?") {
            Ok(true) => (),
            Ok(false) | Err(_) => {
                println!("migration was cancled");
                std::process::exit(1);
            }
        }
    }

    if let Ok(conf) = serde_json::from_str::<OldConfig>(&contents) {
        perform_migration(conf);
    }

    let new_config_path = config_path.with_extension("json.old");
    if let Err(e) = fs::rename(&config_path, &new_config_path) {
        if !matches!(e.kind(), ErrorKind::NotFound) {
            eprintln!("Renaming config Path from {config_path:?} to {new_config_path:?} failed",)
        }
    }
}
fn perform_migration(old: OldConfig) {
    println!("Migrating to new Wechsel setup");

    fn recurse(prj: &OldProject, new_parent_path: &PathBuf, old_parent_path: &PathBuf) {
        let old_prj_folder = path_from_iter([old_parent_path, &PathBuf::from(&prj.path)]);
        let new_prj_folder = path_from_iter([new_parent_path, &PathBuf::from(&prj.name)])
            .with_extension(PROJECT_EXTENSION);

        if !new_prj_folder.exists() || !new_prj_folder.is_dir() {
            fs::create_dir(&new_prj_folder).unwrap_or_else(|e| {
                panic!("Could not create project folder: {new_prj_folder:?} {}", e)
            });
        }

        for folder in prj.folder.iter() {
            let folder_path = path_from_iter([&old_prj_folder, &PathBuf::from(folder)]);

            let new_folder_path = path_from_iter([
                &new_prj_folder,
                &PathBuf::from(PathBuf::from(&folder).file_name().unwrap()),
            ])
            .with_extension(WECHSEL_FOLDER_EXTENSION);

            if !new_folder_path.exists() {
                if let Err(e) = fs::rename(&folder_path, &new_folder_path) {
                    if !matches!(e.kind(), ErrorKind::AlreadyExists) {
                        eprintln!(
                            "Could not move wechsel folder to new project folder {new_folder_path:?}"
                        )
                    }
                }
            }
        }

        for child in &prj.children {
            recurse(child, &new_prj_folder, &old_prj_folder);
        }

        if old_prj_folder.exists() && old_prj_folder.is_dir() {
            for file in fs::read_dir(&old_prj_folder).unwrap() {
                let Ok(file) = file else { continue };
                let new_file = path_from_iter([&new_prj_folder, &PathBuf::from(file.file_name())]);
                if !new_file.exists() {
                    if let Err(e) = fs::rename(file.path(), new_file) {
                        eprintln!(
                            "Could not move file ({:?}) from old prj ({}) to new one {}",
                            file.file_name(),
                            prj.name,
                            e
                        );
                    }
                } else {
                    eprintln!(
                        "Could not move file ({:?}) from old prj ({}): File already exist in the new project",
                        file.file_name(),
                        prj.name
                    );
                }
            }
            // delete old prj folder if its empty now
            if fs::read_dir(&old_prj_folder).unwrap().next().is_none() {
                fs::remove_dir(old_prj_folder).unwrap();
            }
        }
    }

    recurse(
        &old.all_prjs,
        &dirs::home_dir().expect("Could not find home directory"),
        &dirs::home_dir().expect("Could not find home directory"),
    );
}
