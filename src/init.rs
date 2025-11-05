use std::{
    fs::{self, OpenOptions},
    io,
    path::PathBuf,
};

use dialoguer::MultiSelect;
use io::Write;
use std::os::unix::fs::PermissionsExt;

use crate::{
    PROJECT_EXTENSION, WECHSEL_FOLDER_EXTENSION,
    utils::{get_home_folder_paths, path_from_iter},
};

pub const DEFAULT_ROOT_PRJ: &str = "home";

pub fn bashrc_path() -> PathBuf {
    path_from_iter([
        dirs::home_dir().expect("No Home dir found"),
        PathBuf::from(".bashrc"),
    ])
}

pub fn fish_config_path(config_dir: &PathBuf) -> PathBuf {
    path_from_iter([config_dir, &PathBuf::from("fish/config.fish")])
}

pub fn on_prj_change_path(config_dir: &PathBuf) -> PathBuf {
    path_from_iter([config_dir, &PathBuf::from("on-prj-change")])
}

pub fn init_prj(config_dir: PathBuf, no_prompts: bool) -> String {
    println!("Initializing Wechsel");
    let home = dirs::home_dir().expect("Could not find home directory");

    // Check for an existing installation
    let mut found_project_folder = None;
    let mut single_project_exception = true;
    for entry in fs::read_dir(&home).unwrap().filter_map(|entry| entry.ok()) {
        if entry.file_type().map(|typ| typ.is_dir()).unwrap_or(false) {
            let name = entry.file_name();
            let name = name.as_os_str().to_str();

            if name
                .map(|name| name.ends_with(format!(".{PROJECT_EXTENSION}").as_str()))
                .unwrap_or(false)
            {
                if found_project_folder.is_some() {
                    single_project_exception = false;
                }
                found_project_folder.replace(entry.path());
                if !single_project_exception {
                    break;
                }
            } else if name
                .map(|name| name.ends_with(format!(".{WECHSEL_FOLDER_EXTENSION}").as_str()))
                .unwrap_or(false)
            {
                single_project_exception = false;
            }
        }
    }

    let prj_path = match (found_project_folder, single_project_exception) {
        (Some(path), true) => path, // The single project exception means that the root project is the sole project in the home folder and home folder has no .w folders of its own
        (Some(_), false) => home.clone(),
        _ => path_from_iter([&home, &PathBuf::from(DEFAULT_ROOT_PRJ)])
            .with_extension(PROJECT_EXTENSION),
    };

    if !prj_path.exists() {
        println!("Creating root project folder: at {prj_path:?}");
        fs::create_dir_all(&prj_path).expect("Could not create project folder");
    } else {
        println!("root project folder already exists");
    }

    let (home_folder_names, home_folder_paths): (Vec<_>, Vec<_>) = get_home_folder_paths().unzip();

    let folders = if !no_prompts {
        println!();

        println!("Wechsel will now move some of your user folders to the root project.");
        println!("You should select all folders that you want projects to be able to use.");

        let Ok(folders) = MultiSelect::new()
            .with_prompt("Select folders to move to the root project")
            .items(&home_folder_names)
            .report(false)
            .interact()
            .map(|i| i.into_iter().map(|i| home_folder_names[i]).collect())
        else {
            std::process::exit(1);
        };
        folders
    } else {
        home_folder_names.clone()
    };

    for folder in folders.iter() {
        let folder_path = home_folder_paths
            .get(
                home_folder_names
                    .iter()
                    .position(|name| folder == name)
                    .unwrap(),
            )
            .unwrap();

        let target = path_from_iter([&prj_path, &PathBuf::from(folder)])
            .with_extension(WECHSEL_FOLDER_EXTENSION);

        if !folder_path.is_dir() {
            println!("Folder {folder:?} does not exist or isn't a directory");
            continue;
        }

        if target.exists() {
            println!("Folder {folder:?} already exists in root project");
            continue;
        }

        println!("Moving folder {:?} to {:?}", folder_path, &target);
        if let Err(err) = fs::rename(folder_path, &target) {
            eprintln!("Could not move folder: {folder_path:?} to {target:?}, ignoring it; {err}");
        }
    }

    let shells = if !no_prompts {
        println!("Wechsel is now ready to use.");
        println!();
        println!("Would you like to integrate Wechsel into your shells?");

        let items = vec!["Bash", "Fish"];
        let Ok(shells) = MultiSelect::new()
            .with_prompt("Select shells")
            .items(&items)
            .report(false)
            .interact()
            .map(|i| i.into_iter().map(|i| items[i]).collect())
        else {
            std::process::exit(1);
        };
        shells
    } else {
        vec!["Bash", "Fish"]
    };

    if shells.contains(&"Bash") {
        let mut file = OpenOptions::new().append(true).open(bashrc_path());

        if let Ok(file) = &mut file {
            let bash = include_str!("../config_files/default_bash_config");

            for line in bash.lines() {
                if let Err(e) = writeln!(file, "{line}") {
                    eprintln!("Couldn't write to file: {e}");
                }
            }
        } else {
            eprintln!("Couldn't open .bashrc, continuing without modifying it.");
        }
    }

    if shells.contains(&"Fish") {
        let mut file = OpenOptions::new()
            .append(true)
            .open(fish_config_path(&config_dir));

        if let Ok(file) = &mut file {
            let fish = include_str!("../config_files/default_fish_config");
            for line in fish.lines() {
                if let Err(e) = writeln!(file, "{line}") {
                    eprintln!("Couldn't write to file: {e}");
                }
            }
        } else {
            eprintln!("Couldn't open fish config, continuing without modifying it.");
        }
    }

    let on_prj_change = on_prj_change_path(&config_dir);

    if !on_prj_change.exists() {
        println!("Creating on-prj-change script");

        let default = include_str!("../config_files/default_on_prj_change");

        fs::write(&on_prj_change, default).expect("Could not create on-prj-change script");

        let mut permissions = fs::metadata(&on_prj_change)
            .expect("Could not get metadata")
            .permissions();

        // Add execute permission
        permissions.set_mode(permissions.mode() | 0b001001001);
        fs::set_permissions(&on_prj_change, permissions).expect("Could not set permissions");
    } else {
        println!("on-prj-change folder already exists");
    }
    DEFAULT_ROOT_PRJ.to_string()
}
