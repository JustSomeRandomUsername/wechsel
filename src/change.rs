use std::{collections::HashMap, fs, io, path::PathBuf, vec};

pub const CURRENT_PROJECT_FOLDER: &str = "Project";

use crate::{
    init::on_prj_change_path,
    tree::search_for_projects,
    utils::{get_folders, path_from_iter, query_active_project},
};

pub fn get_enviroment_vars_path(config_dir: &PathBuf) -> PathBuf {
    path_from_iter([config_dir, &PathBuf::from("environment_variables.sh")])
}

pub fn get_enviroment_vars_fish_path(config_dir: &PathBuf) -> PathBuf {
    path_from_iter([config_dir, &PathBuf::from("environment_variables.sh")])
}

fn link_folder(path: &PathBuf, target_name: &str) -> io::Result<bool> {
    let target = path_from_iter([
        dirs::home_dir().ok_or(io::Error::new(
            std::io::ErrorKind::Other,
            "No Home dir found",
        ))?,
        PathBuf::from(target_name),
    ]);
    if !path.exists() || (target.exists() && !target.is_symlink()) {
        if !path.exists() {
            println!("Could not symlink folder ({path:?}) because it doesn't exists",);
        }
        println!(
            "Could not symlink folder ({path:?}): {target:?}exists and is not a symlink. Did you already initialize wechsel on your system? Calling `wechsel init` might resolve this issue.",
        );
        return Ok(false);
    }

    if target.is_symlink() {
        fs::remove_file(&target)?;
    }

    std::os::unix::fs::symlink(path, &target).unwrap_or_default();

    Ok(true)
}

pub fn change_prj(prj_name: &str, config_dir: PathBuf) -> io::Result<()> {
    // Find Project Folder Urls

    let active = query_active_project().unwrap_or_default();

    let [prj_path, old_prj_path] = search_for_projects([prj_name, active.as_str()], &config_dir);

    let Some(prj_path) = prj_path else {
        eprintln!("Could not find Project {}", prj_name);
        std::process::exit(1);
    };
    link_folder(&prj_path.path, CURRENT_PROJECT_FOLDER)?;

    let prj_path_string = prj_path.path.to_str().unwrap_or_default().to_string();

    // Link Folders
    let mut prj = Some(prj_path);
    let mut linked_folders = vec![];
    loop {
        let Some(p) = prj else { break };
        for path in get_folders(&p.path) {
            let Some(clean_name) = path
                .file_stem()
                .unwrap()
                .to_str()
                .map(|name| name.to_string())
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
        prj = p.parent.clone();
    }

    let mut env_vars: HashMap<String, String> = HashMap::from_iter(vec![
        ("PRJ".to_owned(), prj_name.to_owned()),
        ("PRJ_PATH".to_owned(), prj_path_string.clone()),
        ("OLD_PRJ".to_owned(), active.clone()),
    ]);

    if let Some(old_prj_path) = old_prj_path {
        let old_prj_path = old_prj_path.path.to_str().unwrap_or_default().to_string();
        env_vars.insert("OLD_PRJ_PATH".to_owned(), old_prj_path);
    }

    // Write Environment Variables for Fish
    let environment_vars = get_enviroment_vars_fish_path(&config_dir);
    fs::write(
        environment_vars,
        format!("set -x PRJ {prj_name}\nset -x PRJ_PATH {prj_path_string}"),
    )?;

    // Write Environment Variables for Bash
    let environment_vars = get_enviroment_vars_path(&config_dir);
    fs::write(
        environment_vars,
        format!("export PRJ={prj_name}\nexport PRJ_PATH={prj_path_string}"),
    )?;

    // Global on change script .config/on-prj-change
    let on_change = on_prj_change_path(&config_dir);

    if on_change.is_file() {
        if let Ok(mut child) = std::process::Command::new("sh")
            .envs(env_vars)
            .arg("-c")
            .arg(&on_change)
            .spawn()
        {
            let _ = child.wait();
        }
    }
    Ok(())
}
