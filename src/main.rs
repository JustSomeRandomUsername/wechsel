use clap::Parser;
use serde::{Deserialize, Serialize};
use core::panic;
use std::collections::HashMap;
use std::{fs, vec, io};
use std::path::PathBuf;
use dirs;

#[derive(Parser, Debug)]
struct Args {
    project_name: Option<String>,

    #[arg(short, long)]
    new: bool,

    #[arg(short, long)]
    initial: bool,

    #[arg(long)]
    remove: bool,

    #[arg(long)]
    rename: bool,
    
    #[clap(long, requires = "rename")]
    new_name: Option<String>,

    #[clap(long, requires = "rename")]
    new_path: Option<String>,

    #[arg(long)]
    path: Option<String>,

    #[arg(long)]
    parent: Option<String>,

    #[clap(long, value_parser, num_args = 1.., value_delimiter = ' ')]
    folders: Option<Vec<String>>,
}

#[derive(Default, Deserialize, Serialize, Debug)]
struct Config {
    active: String,
    all_prjs: Project
}

#[derive(Default, Deserialize, Serialize, Debug)]
struct Project {
    name: String,
    path: String,
    folder: Vec<String>,
    children: Vec<Project>
}

const CONFIG_NAME: &str = "prj-settings.json";

fn main() {
    let args = Args::parse();
    println!("{:?}", args);

    let mut path = PathBuf::new();
    path.push(dirs::config_dir().expect("No config folder found"));
    path.push(CONFIG_NAME);

    let (mut config, mut prj_name) = if !path.exists() {
        if !args.initial {
            panic!("No config file, you might want to call this script with --inital")
        }
        let (conf, name) = init_prj(&args).expect("Could not create initial project");
        (conf, Some(name))
    
    } else {
        let contents = fs::read_to_string(&path)
            .expect("Should have been able to read the file");

        (serde_json::from_str(&contents).expect("JSON was not well-formatted"), 
            args.project_name.as_ref().map(|s| s.clone()))
    };

    
    if prj_name.is_none() {
        println!("No project name given");
        std::process::exit(1);
    }
    
    let folders = args.folders.clone().unwrap_or(vec!["Desktop".to_owned(), "Downloads".to_owned()]);

    if args.initial {
        //Dont jumpt into new, remove ..

    } else if args.new {
        //New Project
        if let Some(name) = prj_name.as_ref() {
            new_prj(&args, &mut config, name, folders).expect("Could not create new project");
        }
    } else if args.remove {
        //Remove Project
        if let Some(name) = prj_name.as_ref() {    
            println!("Removing Project {}, folders will not be deleted", name);
    
            config.all_prjs.walk(name, name, |_,_| {},
                |p, target,_,_|{
                let index = p.children.iter()
                    .position(|pr| &pr.name == target)
                    .expect("Could not find project to remove");
                p.children.remove(index);
            });
        }
        if prj_name.is_some() && prj_name.as_ref().unwrap() == &config.active {
            prj_name = Some(config.all_prjs.name.clone());   
        }
    } else if args.rename {

        let new_name = args.new_name.as_ref().unwrap_or(args.project_name.as_ref().unwrap());
        // let new_path = args.new_path.as_ref().unwrap_or(default)
        let from = config.all_prjs.walk(prj_name.as_ref().unwrap(), &(new_name, args.new_path.as_ref()),
                |p, new_data| {
                    let old_path = PathBuf::from(&p.path);
                    p.name = new_data.0.clone();
                    if let Some(new_path) = new_data.1 {
                        p.path = new_path.clone();
                    }
                    old_path
                },
                |p, _, child_path, _| PathBuf::from_iter(vec![PathBuf::from(&p.path), child_path]))
                .expect("Could not find project path");

            if let Some(new_path) = args.new_path.as_ref() {
                let mut to = from.clone();
                to.pop();
                to.push(new_path);
                fs::rename(from, to)
                    .expect("Could not rename");
            }

            prj_name = Some(new_name.clone());
    }

    //Change Project
    if let Some(target) = prj_name.as_ref() {
        match change_prj(&target, &mut config) {
            Ok(_) => println!("Changed to Project {}", target),
            Err(e) => println!("Could not change to Project {}, Error: {}", target, e)
        }
    }

    //Write Config to json
    std::fs::write(&path,serde_json::to_string_pretty(&config).unwrap())
            .expect("Could not write initial config file");
}

fn init_prj(args: &Args) -> io::Result<(Config, String)> {
    let name = args.project_name.as_ref().map(|s|s.as_str()).unwrap_or("default");

    let prjs_path = args.path
        .as_ref()
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|| {
            println!("No path given, using default");
            let mut path = dirs::home_dir().expect("No Home dir found");
            path.push("projects");
            path
        });

    
    println!("Creating initial project folder in {:?} with name {}", prjs_path, name);
    let mut prj_path = prjs_path.clone();
    prj_path.push(name);
    
    let folders = args.folders.clone().unwrap_or(vec!["Desktop", "Downloads", "Documents", "Pictures", "Videos", "Music"]
                                                                    .iter().map(|s| s.to_string()).collect::<Vec<String>>());

    println!("Folders that will be moved from home to the new project folder: {:?}", folders);

    if !prj_path.exists() {
        println!("Creating project folder");
        fs::create_dir_all(&prj_path)
            .expect("Could not create project folder");
    } else {
        println!("Project folder already exists");
    }
    println!("Moving folders {:?}", prj_path);

    for folder in &folders {
        let folder_path = match folder.as_str() {
            "Desktop" => dirs::desktop_dir().expect("Desktop dir not found"),
            "Documents" => dirs::document_dir().expect("Documents dir not found"),
            "Downloads" => dirs::download_dir().expect("Downloads dir not found"),
            "Pictures" => dirs::picture_dir().expect("Pictures dir not found"),
            "Videos" => dirs::video_dir().expect("Videos dir not found"),
            "Music" => dirs::audio_dir().expect("Music dir not found"),
            a => PathBuf::from(a)
        };

        let mut target = prj_path.clone();
        target.push(folder);
        println!("Moving folder {:?} to {:?}", folder_path, &target);
        fs::rename(&folder_path, &target)
            .expect(format!("Could not move folder: {}", folder).as_str());
    }
        
    let conf = Config {
        active: String::new(),
        all_prjs: Project {
            name: name.to_owned(),
            path: prjs_path.to_str().unwrap().to_owned(),
            folder: folders.clone().into_iter().map(|s| format!("{}/{}",name,s)).collect::<Vec<String>>(),
            children: vec![]
        }
    };
    
    Ok((conf, name.to_string()))
}

fn new_prj(args: &Args, config: &mut Config, prj_name: &str, folders: Vec<String>) -> io::Result<()> {
    println!("Creating Project {:?}", args.project_name);
        
    let parent = args.parent.as_ref().map(|s|s.clone()).unwrap_or(config.all_prjs.name.clone());

    //get parent url
    let parent_url = config.all_prjs.walk(&parent, &(),
        |p, _| p.path.clone(), 
        |p, _, child_path, _| format!("{}/{}", p.path, child_path))
        .expect("Could not find parent path");

    println!("Parent url: {:?}", parent_url);

    //Use Project Name as Path if no path is given
    let mut new_pr_path = PathBuf::from(&parent_url);

    if let Some(path) = args.path.as_ref() {
        new_pr_path.push(path);
    } else {
        new_pr_path.push(prj_name);
    }

    //Create Project Folder
    if !new_pr_path.exists() {
        fs::create_dir(&new_pr_path)
           .expect(format!("Could not create project folder: {}", new_pr_path.to_str().unwrap()).as_str());
    }

    //Create Subfolders
    for subfolder in folders.iter() {
        new_pr_path.push(subfolder);
        
        fs::create_dir(&new_pr_path)
            .expect(format!("Could not create Project Subfolder: {}", new_pr_path.to_str().unwrap()).as_str());

        new_pr_path.pop();
    }
    
    //Insert new Project into config
    config.all_prjs.walk(&parent, &(prj_name, folders),
        |p, (name, folders)| {
            p.children.push(Project {
                name: name.to_string(),
                path: name.to_string(),
                folder: folders.clone(),
                children: vec![] });
        }, 
        |_, _, _, _| ());
    Ok(())
}

fn change_prj(name: &str, config: &mut Config) -> io::Result<()> {
    // Find Project Folder Urls
    let map = config.all_prjs.walk(name, &(),
    |p, _| {
        let new_links = p.folder.iter()
            .map(|f| 
                (PathBuf::from(f).file_name().unwrap().to_str().unwrap().to_owned(), PathBuf::from_iter(vec![&p.path, &f])))
            .collect::<HashMap<String, PathBuf>>();
        
        return (new_links, p.path.clone());
    },
    |p,_,child_links: (HashMap<String, PathBuf>, String),gen| {
        let mut links = gen(p,&()).0;
        links.extend(child_links.0.iter()
            .map(|(k,v)| {
                let mut path = PathBuf::from(&p.path);
                path.push(v);
                return (k.clone(), path);
            }).collect::<HashMap<String, PathBuf>>());

        return (links, format!("{}/{}",p.path, child_links.1));
    });

    if let Some((map, prj_path)) = map {
        // Link Folders
        for (name, path) in map {
            let mut parking = dirs::home_dir().ok_or(io::Error::new(std::io::ErrorKind::Other, "No Home dir found"))?;
            let mut target = parking.clone();
            target.push(&name);
                
            if !path.exists() || (target.exists() && !target.is_symlink()) {
                println!("Not found Path {:?}, target {:?} {:?}", path, parking, target);
                continue;   
            }
            parking.push(name+"1"); 

            std::os::unix::fs::symlink(&path, &parking).unwrap_or_default();

            fs::rename(&parking, &target)?
            
        }
        // Link init Script
        if let Some(mut script_target) = dirs::home_dir() {
            script_target.push(".init-prj");
            if script_target.exists() {
                fs::remove_file(&script_target)?
            }

            let mut script_path = PathBuf::from(&prj_path);
            script_path.push(".init-prj");
            if script_path.is_file() {
                std::os::unix::fs::symlink(&script_path, &script_target).unwrap_or_default();
            }
        }

        fn execute(script: PathBuf) -> io::Result<()>{
            if script.is_file() {
                std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&script)
                    .spawn()?;
            }
            Ok(())
        }
        let mut on_change_script = PathBuf::from(&prj_path);
        on_change_script.push(".on-prj-change");
        execute(on_change_script)?;

        if let Some(mut script) = dirs::home_dir() {
            script.push(".on-prj-change");
            execute(script)?;
        }
    }
        
    config.active = name.to_owned();
    return Ok(());
}


impl Project {
    fn walk<T, Z>(&mut self, target: &str, data: &Z, gen: fn(&mut Project, &Z) -> T, res: fn(&mut Project, &Z, T, fn(&mut Project, &Z) -> T) -> T) -> Option<T> {
        if self.name == target {
            return Some(gen(self, data));
        }
        let child_data = self.children.iter_mut()
            .map(|pr| pr.walk(target, data, gen, res))
            .find(|op| op.is_some());

        if let Some(child_path) = child_data.flatten() {
            return Some(res(self, data, child_path, gen));
        };
        None
    }
}