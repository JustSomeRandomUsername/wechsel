use std::{fs::{self, OpenOptions}, io, path::PathBuf};

use inquire::{MultiSelect, Text};
use io::Write;
use std::os::unix::fs::PermissionsExt;

use crate::{Config, Project, CONFIG_NAME};

pub fn init_prj(config_dir: PathBuf) -> io::Result<(Config, String)> {
    
    println!("Wechsel need a folder to store your projects in.");
    println!("By default it will be called \"projects\" and will be created in your home directory.");
    println!("You can pick where you want to have it, either enter an absolute path or your input will be used relative to you home directory.");
    let Ok(name) = Text::new("projects folder path")
        .with_default("projects")
        .prompt() else {
            std::process::exit(1);
        };

    let prjs_path = dirs::home_dir()
        .expect("No Home dir found")
        .join(name.clone());
    
    if !prjs_path.exists() {
        println!("Creating project folder");
        fs::create_dir_all(&prjs_path)
            .expect("Could not create project folder");
    } else {
        println!("projects folder already exists");
    }
    println!();
    println!("Wechsel uses a tree structure to organise your projects.");
    println!("This will now create the root project.");
    let Ok(name) = Text::new("Name of the root project")
        .with_default("default")
        .prompt() else {
            std::process::exit(1);
        };

    
    let mut root_prj = prjs_path.clone();
    root_prj.push(name.clone());
    
    if !root_prj.exists() {
        println!("creating root project folder called \"{name}\"");
        fs::create_dir_all(&root_prj)
          .expect("Could not create project folder");
    } else {
        println!("root project folder already exists");
    }

    println!();

    println!("Wechsel will now move some of your user folders to the default project.");
    println!("You should select all folders that you want any project to be able to use.");
    println!("Otherwise Wechsel cant create symlinks in your home folder.");
    let Ok(folders) = MultiSelect::new("Select folders to move to the root project", 
            vec!["Desktop", "Downloads", "Documents", "Pictures", "Videos", "Music"])
        .with_default(&[0,1,2,3,4,5])
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
            a => PathBuf::from(a)
        };

        let mut target = root_prj.clone();
        target.push(folder);

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

    let path: PathBuf = [
        config_dir.clone(),
        PathBuf::from(CONFIG_NAME)
    ].iter().collect();
    println!("Wechsel will now create a settings file in at {path:?}");
        
    let conf = Config {
        active: String::new(),
        all_prjs: Project {
            name: name.to_owned(),
            path: prjs_path.to_str().unwrap().to_owned(),
            folder: folders.clone().into_iter().map(|s| {
                let folder: PathBuf = [&name, s].iter().collect();
                folder.to_str().expect("Could not convert to string").to_owned()
        }).collect::<Vec<String>>(),
            children: vec![]
        }
    };
    
    println!("Wechsel is now ready to use.");
    println!();
    println!("Would you like to integrate Wechsel into your shells?");

    let Ok(shells) = MultiSelect::new("Select shells", 
        vec!["Bash", "Fish"])
        .with_default(&[0,1])
        .prompt() else {
            std::process::exit(1);
        };

    if shells.contains(&"Bash") {
        let mut bashrc = dirs::home_dir().expect("No Home dir found");
        bashrc.push(".bashrc");

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(bashrc)
            .unwrap();

        let bash = 
"env_vars=~/.config/wechsel/enviroment_variables.sh
if [ -f $env_vars ]; then
    . $env_vars
fi
init=$PRJ_PATH/.init-prj
if [ -f $init ]; then
    . $init
fi";

        for line in bash.lines() {
            if let Err(e) = writeln!(file, "{}", line) {
                eprintln!("Couldn't write to file: {}", e);
            }
        }
    }

    if shells.contains(&"Fish") {
        let mut fish_config = dirs::config_dir().expect("No Home dir found");
        fish_config.push("fish/config.fish");

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(fish_config)
            .unwrap();

        let fish = 
"if status is-interactive
    set env_var ~/.config/wechsel/enviroment_variables.fish
    if test -e $env_var
        source $env_var 
    end
    set init $PRJ_PATH/.init-prj.fish
    if test -e $init
        source $init
    end
end";
        for line in fish.lines() {
            if let Err(e) = writeln!(file, "{}", line) {
                eprintln!("Couldn't write to file: {}", e);
            }
        }
    }
    let mut on_prj_change = config_dir.clone();
    on_prj_change.push("on-prj-change");

    if !on_prj_change.exists() {
        println!("Creating on-prj-change script");

        let default = 
"
#!/bin/bash

#Reload Indicator
gdbus call -e -d org.gnome.shell.extensions.prjchange -o /org/gnome/shell/extensions/prjchange/service -m org.gnome.shell.extensions.prjchange.service.Reload > /dev/null;\\

#Reload Nautilus Windows
windows=$(gdbus introspect -e -d org.gnome.Nautilus -o /org/gnome/Nautilus -r | grep -Po 'window/\\K[0-9]*')
for i in $windows;do \\
	gdbus call -e -d org.gnome.Nautilus -o /org/gnome/Nautilus/window/$i -m org.gtk.Actions.Activate \"@s 'reload'\" \"@av []\" \"@a{sv} {}\" > /dev/null;\\
done	

#Reload Ding Desktop Icons
#gdbus call -e -d org.gnome.Nautilus -o /org/gnome/Nautilus/window/$i -m org.gtk.Actions.Activate \"@s 'reload'\" \"@av []\" \"@a{sv} {}\" > /dev/null;\\

# Set gnome wallpapers
# This is searching for images named wallpaper.png and wallpaper_dark.png
# Light mode wallpaper
#wallpaper=$PRJ_PATH/wallpaper.png
#if [ -f $wallpaper  ]; then
#	gsettings set org.gnome.desktop.background picture-uri file://$wallpaper
#else
#	gsettings set org.gnome.desktop.background picture-uri file:///usr/share/backgrounds/f38/default/f38-01-day.png
#fi

# Dark mode wallpaper
#wallpaper=$PRJ_PATH/wallpaper_dark.png
#if [ -f $wallpaper ]; then
#	gsettings set org.gnome.desktop.background picture-uri-dark file://$wallpaper
#else
#	gsettings set org.gnome.desktop.background picture-uri-dark file:///usr/share/backgrounds/f38/default/f38-01-night.png
#fi

# Call project specific on-prj-change
on_change=$PRJ_PATH/.on-prj-change
if [ -f $on_change ]; then
	$on_change
fi";

        fs::write(&on_prj_change, default)
            .expect("Could not create on-prj-change script");

        let mut permissions = fs::metadata(&on_prj_change)
            .expect("Could not get metadata")
            .permissions();

        // Add execute permission
        permissions.set_mode(permissions.mode() | 0b001001001);
        fs::set_permissions(&on_prj_change, permissions)
            .expect("Could not set permissions");

    } else {
        println!("on-prj-change folder already exists");
    }    

    Ok((conf, name.to_string()))
}