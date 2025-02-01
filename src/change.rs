use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use crate::{
    utils::{get_folders, search_for_project},
    Config, FOLDER_PREFIX,
};

fn link_folder(path: &PathBuf, target_name: &str) -> io::Result<bool> {
    let mut target = dirs::home_dir().ok_or(io::Error::new(
        std::io::ErrorKind::Other,
        "No Home dir found",
    ))?;

    target.push(target_name);

    if !path.exists() || (target.exists() && !target.is_symlink()) {
        println!(
            "Could not symlink folder, either {:?} doesn't exists or target {:?} exists and is not a symlink",
            path, target
        );
        return Ok(false);
    }

    if target.is_symlink() {
        fs::remove_file(&target)?;
    }

    std::os::unix::fs::symlink(path, &target).unwrap_or_default();

    Ok(true)
}

pub fn change_prj(
    prj_name: &str,
    config: &mut Config,
    config_dir: PathBuf,
) -> io::Result<(Vec<PathBuf>, HashMap<String, String>)> {
    // Find Project Folder Urls

    let Some(prj_path) = search_for_project(config.base_folder.clone(), prj_name) else {
        eprintln!("Could not find Project {}", prj_name);
        std::process::exit(1);
    };

    link_folder(&prj_path[0], "Project")?;

    // Link Folders
    let mut linked_folders = vec![];
    for level in prj_path.iter() {
        for path in get_folders(level) {
            let Some(clean_name) = path
                .file_stem()
                .unwrap()
                .to_str()
                .map(|name| name.to_string().replace(FOLDER_PREFIX, ""))
            else {
                continue;
            };
            if linked_folders.contains(&clean_name) {
                continue;
            }

            if !link_folder(&path, &clean_name)? {
                continue;
            }

            linked_folders.push(clean_name);
        }
    }

    let prj_path = prj_path[0].to_str().unwrap_or_default().to_string();

    let mut env_vars = HashMap::from_iter(vec![
        ("PRJ".to_owned(), prj_name.to_owned()),
        ("PRJ_PATH".to_owned(), prj_path.clone()),
        ("OLD_PRJ".to_owned(), config.active.clone()),
    ]);

    if let Some(old_prj_path) = search_for_project(config.base_folder.clone(), &config.active) {
        let old_prj_path = old_prj_path[0].to_str().unwrap_or_default().to_string();
        env_vars.insert("OLD_PRJ_PATH".to_owned(), old_prj_path);
    }

    // Write Enviroment Variables for Fish
    let mut enviroment_vars = PathBuf::from(&config_dir);
    enviroment_vars.push("enviroment_variables.fish");
    fs::write(
        enviroment_vars,
        format!("set -x PRJ {prj_name}\nset -x PRJ_PATH {prj_path}"),
    )?;

    // Write Enviroment Variables for Bash
    let mut enviroment_vars = PathBuf::from(&config_dir);
    enviroment_vars.push("enviroment_variables.sh");
    fs::write(
        enviroment_vars,
        format!("export PRJ={prj_name}\nexport PRJ_PATH={prj_path}"),
    )?;

    let mut scripts = vec![];
    // Global on change script .config/on-prj-change
    let mut script = PathBuf::from(&config_dir);
    script.push("on-prj-change");
    scripts.push(script);

    config.active = prj_name.to_owned();
    Ok((scripts, env_vars))
}
