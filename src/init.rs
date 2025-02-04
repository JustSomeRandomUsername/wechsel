use std::{
    fs::{self, OpenOptions},
    io,
    path::PathBuf,
};

use inquire::{MultiSelect, Text};
use io::Write;
use std::os::unix::fs::PermissionsExt;

use crate::{
    utils::{path_from_iter, HOME_FOLDERS},
    Config, CONFIG_NAME, FOLDER_PREFIX,
};

pub fn init_prj(config_dir: PathBuf) -> io::Result<Config> {
    println!("Wechsel uses a tree structure to organise your projects.");
    let Ok(root_prj_name) = Text::new("Name of the root project")
        .with_default("home")
        .prompt()
    else {
        std::process::exit(1);
    };

    println!();

    println!("Next you need to decide where you would like your root project to be located.");
    println!("By default it will be created in your home directory.");
    let Ok(prjs_path) = Text::new("projects folder path")
        .with_default(
            dirs::home_dir()
                .and_then(|a| a.to_str().map(|a| a.to_string()))
                .expect("No Home dir found")
                .as_str(),
        )
        .prompt()
    else {
        std::process::exit(1);
    };
    let prj_path = path_from_iter([prjs_path, root_prj_name]);

    if !prj_path.exists() {
        println!("Creating root project folder");
        fs::create_dir_all(&prj_path).expect("Could not create project folder");
    } else {
        println!("root project folder already exists");
    }

    println!();

    println!("Wechsel will now move some of your user folders to the default project.");
    println!("You should select all folders that you want any project to be able to use.");
    println!("Otherwise Wechsel cant create symlinks in your home folder.");
    let Ok(folders) = MultiSelect::new(
        "Select folders to move to the root project",
        HOME_FOLDERS.to_vec(),
    )
    .with_default(&[0, 1, 2, 3, 4, 5])
    .prompt() else {
        std::process::exit(1);
    };

    for folder in folders.iter() {
        let folder_path = match *folder {
            "Desktop" => dirs::desktop_dir().expect("Desktop dir not found"),
            "Documents" => dirs::document_dir().expect("Documents dir not found"),
            "Downloads" => dirs::download_dir().expect("Downloads dir not found"),
            "Pictures" => dirs::picture_dir().expect("Pictures dir not found"),
            "Videos" => dirs::video_dir().expect("Videos dir not found"),
            "Music" => dirs::audio_dir().expect("Music dir not found"),
            a => PathBuf::from(a),
        };

        let folder_path_buf = PathBuf::from(folder);
        let target = path_from_iter([&prj_path, &folder_path_buf]).with_file_name(
            folder_path_buf
                .file_name()
                .map(|name| format!("{FOLDER_PREFIX}{name:?}"))
                .unwrap_or_default(),
        );

        if !folder_path.is_dir() {
            println!("Folder {folder:?} does not exist or isn't a directory");
            continue;
        }

        if target.exists() {
            println!("Folder {folder:?} already exists in root project");
            continue;
        }

        println!("Moving folder {:?} to {:?}", folder_path, &target);
        if fs::rename(&folder_path, &target).is_err() {
            eprintln!("Could not move folder: {folder_path:?}, ignoring it");
        }
    }

    let config_path = path_from_iter([config_dir.clone(), PathBuf::from(CONFIG_NAME)]);
    println!("Wechsel will now create a settings file in at {config_path:?}");

    let conf = Config {
        active: String::new(),
        base_folder: prj_path,
    };

    println!("Wechsel is now ready to use.");
    println!();
    println!("Would you like to integrate Wechsel into your shells?");

    let Ok(shells) = MultiSelect::new("Select shells", vec!["Bash", "Fish"])
        .with_default(&[0, 1])
        .prompt()
    else {
        std::process::exit(1);
    };

    if shells.contains(&"Bash") {
        let bashrc = path_from_iter([
            dirs::home_dir().expect("No Home dir found"),
            PathBuf::from(".bashrc"),
        ]);

        let mut file = OpenOptions::new().append(true).open(bashrc);

        if let Ok(file) = &mut file {
            let bash = include_str!("../config_files/default_bash_config");

            for line in bash.lines() {
                if let Err(e) = writeln!(file, "{}", line) {
                    eprintln!("Couldn't write to file: {}", e);
                }
            }
        } else {
            eprintln!("Couldn't open .bashrc, continuing without modifying it.");
        }
    }

    if shells.contains(&"Fish") {
        let fish_config = path_from_iter([
            dirs::config_dir().expect("No Home dir found"),
            PathBuf::from("fish/config.fish"),
        ]);

        let mut file = OpenOptions::new().append(true).open(fish_config);

        if let Ok(file) = &mut file {
            let fish = include_str!("../config_files/default_fish_config");
            for line in fish.lines() {
                if let Err(e) = writeln!(file, "{}", line) {
                    eprintln!("Couldn't write to file: {}", e);
                }
            }
        } else {
            eprintln!("Couldn't open fish config, continuing without modifying it.");
        }
    }
    let on_prj_change = path_from_iter([config_dir.clone(), PathBuf::from("on-prj-change")]);

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

    Ok(conf)
}
