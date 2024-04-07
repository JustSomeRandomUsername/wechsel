use std::{fs, io, path::PathBuf};


use crate::{Args, Config, Project};

pub fn init_prj(args: &Args) -> io::Result<(Config, String)> {



    // let name = args.project_name.as_ref().map(|s|s.as_str()).unwrap_or("default");

    // let prjs_path = args.path
    //     .as_ref()
    //     .map(|s| PathBuf::from(s))
    //     .unwrap_or_else(|| {
    //         println!("No path given, using default");
    //         let mut path = dirs::home_dir().expect("No Home dir found");
    //         path.push("projects");
    //         path
    //     });

    
    // println!("Creating initial project folder in {:?} with name {}", prjs_path, name);
    // let mut prj_path = prjs_path.clone();
    // prj_path.push(name);
    
    // let folders = args.folders.clone().unwrap_or(vec!["Desktop", "Downloads", "Documents", "Pictures", "Videos", "Music"]
    //                                                                 .iter().map(|s| s.to_string()).collect::<Vec<String>>());

    // println!("Folders that will be moved from home to the new project folder: {:?}", folders);

    // if !prj_path.exists() {
    //     println!("Creating project folder");
    //     fs::create_dir_all(&prj_path)
    //         .expect("Could not create project folder");
    // } else {
    //     println!("Project folder already exists");
    // }
    // println!("Moving folders {:?}", prj_path);

    // for folder in &folders {
    //     let folder_path = match folder.as_str() {
    //         "Desktop" => dirs::desktop_dir().expect("Desktop dir not found"),
    //         "Documents" => dirs::document_dir().expect("Documents dir not found"),
    //         "Downloads" => dirs::download_dir().expect("Downloads dir not found"),
    //         "Pictures" => dirs::picture_dir().expect("Pictures dir not found"),
    //         "Videos" => dirs::video_dir().expect("Videos dir not found"),
    //         "Music" => dirs::audio_dir().expect("Music dir not found"),
    //         a => PathBuf::from(a)
    //     };

    //     let mut target = prj_path.clone();
    //     target.push(folder);
    //     println!("Moving folder {:?} to {:?}", folder_path, &target);
    //     fs::rename(&folder_path, &target)
    //         .expect(format!("Could not move folder: {}", folder).as_str());//TODO should not panic if folder not found
    // }
        
    // let conf = Config {
    //     active: String::new(),
    //     all_prjs: Project {
    //         name: name.to_owned(),
    //         path: prjs_path.to_str().unwrap().to_owned(),
    //         folder: folders.clone().into_iter().map(|s| format!("{}/{}",name,s)).collect::<Vec<String>>(),
    //         children: vec![]
    //     }
    // };
    
    // Ok((conf, name.to_string()))
    Ok((Config::default(), "default".to_string()))
}