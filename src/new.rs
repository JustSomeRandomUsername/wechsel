use crate::{
    utils::{path_from_iter, search_for_project},
    Config, FOLDER_PREFIX, PROJECTS_FOLDER,
};
use std::{collections::HashMap, fs, io, path::PathBuf};

pub fn new_prj(
    config: &mut Config,
    prj_name: &str,
    folders: Vec<String>,
    parent: String,
    config_dir: &PathBuf,
) -> io::Result<()> {
    println!("Creating Project {:?}", prj_name);

    //get parent path
    let mut parent_path =
        search_for_project(config.base_folder.clone(), &parent).unwrap_or_else(|| {
            eprintln!("The given parent project could not be found");
            std::process::exit(1);
        });

    let mut new_pr_path = path_from_iter([
        parent_path.remove(0),
        PathBuf::from(PROJECTS_FOLDER),
        PathBuf::from(prj_name),
    ]);

    // Create Project Folder

    if !new_pr_path.exists() {
        fs::create_dir_all(&new_pr_path).expect("Could not create Project Folder");
    } else if !new_pr_path.is_dir() {
        eprintln!("A file with the name of the new project exists in the place the project folder should be placed. Please either remove that file or specify a different name. {new_pr_path:?}")
    }

    // Create Subfolders
    for subfolder in folders.iter() {
        new_pr_path.push(format!("{FOLDER_PREFIX}{subfolder}"));

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
    let mut script = PathBuf::from(config_dir);
    script.push("on-prj-create");
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
            .current_dir(new_pr_path)
            .spawn()
        {
            let _ = child.wait();
        }
    }
    Ok(())
}
