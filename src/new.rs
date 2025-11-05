use dialoguer::{Input, MultiSelect};

use crate::{
    PROJECT_EXTENSION, WECHSEL_FOLDER_EXTENSION, query_active_project,
    tree::{ProjectTreeNode, get_project_tree, search_for_projects},
    utils::path_from_iter,
};
use std::{collections::HashMap, fs, io, path::PathBuf};

pub fn new_prj_cmd(
    parent: Option<String>,
    folders: Option<Vec<String>>,
    project_name: &str,
    config_dir: &PathBuf,
) {
    let pwd = std::env::current_dir().expect("Could not get current dir");

    // Check if the pwd folder is a project folder
    let found_parent = parent
        .is_none()
        .then(|| {
            pwd.extension()
                .map(|ext| ext == PROJECT_EXTENSION)
                .unwrap_or_default()
                .then(|| {
                    // current dir is a project
                    pwd.file_name()
                        .and_then(|name| name.to_str())
                        .expect("Could not get parent folder name")
                        .to_string()
                })
        })
        .flatten();

    let (parent, folders) = if parent.is_none() && folders.is_none() {
        // If no options are set, ask for them interactively

        let parent = found_parent.unwrap_or_else(|| {
            // not in a wechsel project so the user has to supply a parent

            let Ok(parent) = Input::new().with_prompt("Parent project").interact_text() else {
                std::process::exit(1);
            };
            parent
        });

        //TODO Should check if parent exists

        fn collect_folders(folders: &mut Vec<String>, node: ProjectTreeNode) {
            if let Some(current_folders) = node.folders {
                for folder in current_folders {
                    if !folders.iter().any(|f| f == &folder) {
                        folders.push(folder);
                    }
                }
            }
            for child in node.children {
                collect_folders(folders, child);
            }
        }
        let mut folders = vec![];
        collect_folders(&mut folders, get_project_tree(config_dir, true));

        let Ok(folders) = MultiSelect::new()
            .with_prompt("Select folders to move to the new project")
            .items(&folders)
            .report(false)
            .interact()
            .map(|i| {
                i.into_iter()
                    .map(|i| folders[i].clone())
                    .collect::<Vec<_>>()
            })
        else {
            std::process::exit(1);
        };

        let folders: Vec<String> = folders
            .into_iter()
            .map(|folder| folder.to_owned())
            .collect();

        (parent, folders)
    } else {
        (
            parent.unwrap_or(query_active_project().unwrap_or_default()),
            folders.unwrap_or(vec!["Desktop".to_owned(), "Downloads".to_owned()]),
        )
    };

    create_new_prj(project_name, folders, parent, config_dir)
        .expect("Could not create new project");
}

pub fn create_new_prj(
    prj_name: &str,
    folders: Vec<String>,
    parent: String,
    config_dir: &PathBuf,
) -> io::Result<()> {
    println!("Creating Project {prj_name:?}");

    //get parent path
    let [parent_path] = search_for_projects([&parent], config_dir);
    let parent_path = &parent_path
        .as_ref()
        .unwrap_or_else(|| {
            eprintln!("The given parent project could not be found");
            std::process::exit(1);
        })
        .path;

    let mut new_pr_path =
        path_from_iter([parent_path, &PathBuf::from(prj_name)]).with_extension(PROJECT_EXTENSION);

    // Create Project Folder
    if !new_pr_path.exists() {
        fs::create_dir_all(&new_pr_path).expect("Could not create Project Folder");
    } else if !new_pr_path.is_dir() {
        eprintln!(
            "A file with the name of the new project exists in the place the project folder should be placed. Please either remove that file or specify a different name. {new_pr_path:?}"
        )
    }

    // Create Subfolders
    for subfolder in folders.iter() {
        new_pr_path.push(PathBuf::from(subfolder).with_extension(WECHSEL_FOLDER_EXTENSION));

        if !new_pr_path.exists() {
            fs::create_dir(&new_pr_path).unwrap_or_else(|e| {
                panic!(
                    "Could not create Project Subfolder: {} {}",
                    new_pr_path.to_str().unwrap(),
                    e
                )
            });
        }

        new_pr_path.pop();
    }

    // Call on create script
    let script = path_from_iter([config_dir, &PathBuf::from("on-prj-create")]);
    if script.is_file() {
        let env_vars: HashMap<String, String> = HashMap::from_iter(vec![
            ("PRJ".to_owned(), prj_name.to_owned()),
            (
                "PRJ_PATH".to_owned(),
                new_pr_path.to_str().unwrap_or_default().to_owned(),
            ),
        ]);
        if let Ok(mut child) = std::process::Command::new("sh")
            .envs(env_vars)
            .arg("-c")
            .arg(&script)
            .current_dir(&new_pr_path)
            .spawn()
        {
            let _ = child.wait();
        }
    }
    Ok(())
}
