use clap::{Parser, Subcommand};
use init::init_prj;
use inquire::{MultiSelect, Text};
use new::new_prj;
use serde::{Deserialize, Serialize};
use core::panic;
use std::collections::HashMap;
use std::{fs, vec};
use std::path::PathBuf;
use dirs;
use crate::change::change_prj;

mod change;
mod new;
mod init;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    command: Option<SubCommands>,

    project_name: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum SubCommands {
    #[clap(about = "Create a new project")]
    New {
        /// parent project
        #[clap(short, long)]
        parent: Option<String>,
        /// relative path from the parent to the new project
        #[clap(long)]
        path: Option<String>,
        /// folders to create in the new project
        #[clap(long, value_parser, num_args = 1.., value_delimiter = ' ')]
        folders: Option<Vec<String>>,
    },
    #[clap(about = "Initialize the config file and create a default project and move the folders to it")]
    Init,
    #[clap(about = "Remove a project from the config file")]
    Remove,
    #[clap(about = "Rename a project")]
    Rename {
        /// new name of the project
        #[clap(long)]
        new_name: Option<String>,
        /// new path of the project (relative path from the parent project)
        #[clap(long)]
        new_path: Option<String>,
    },
    #[clap(about = "Get the path of a project")]
    GetPath,    
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

const CONFIG_NAME: &str = "wechsel_projects.json";

fn main() {    
    let args = Args::parse();
    
    let config_dir: PathBuf = PathBuf::from_iter([
        dirs::config_dir().expect("No config folder found"), 
        PathBuf::from("wechsel")
    ]);
    if !config_dir.exists() {
        fs::create_dir(&config_dir)
            .expect("Could not create config folder");
    }

    let path: PathBuf = [
        config_dir.clone(),
        PathBuf::from(CONFIG_NAME)
    ].iter().collect();

    let (mut config, mut prj_name) =
        if let Some(SubCommands::Init) = args.command {
            let (conf, name) = init_prj(config_dir.clone()).expect("Could not create initial project");

            if path.exists() {
                println!("Config file already exists, keeping the old one");
                (None, None)
            } else {
                (Some(conf), Some(name))
            }
        } else {
            (None, None)
        };
    
    if config.is_none() {
        // Load Config
        if !path.exists() {
            panic!("No config file, you might want to call this script with \"inital\"")
        }
        let contents = fs::read_to_string(&path)
           .expect("Should have been able to read the file");

        config = serde_json::from_str(&contents).expect("JSON was not well-formatted");
        prj_name = args.project_name.as_ref().cloned();
    }

    let mut config = config.unwrap();
    
    if prj_name.is_none() {
        eprintln!("No project name given");
        std::process::exit(1);
    }
    let mut prj_name = prj_name.unwrap();
    if let Some(cmd) = &args.command {
        match cmd {
            SubCommands::New {ref parent, ref path, ref folders} => {
                let (parent, path, folders) =
                    if parent.is_none() && path.is_none() && folders.is_none() {
                        // If no options are set, ask for them interactivly
                        
                        let Ok(parent) = Text::new("parent project")
                            .with_default(&config.all_prjs.name)
                            .prompt() else {
                                std::process::exit(1);
                            };

                        println!("The path of project folder, either relative to the parent project or absolute");
                        let Ok(path) = Text::new("project path")
                            .with_default(&prj_name)
                            .prompt() else {
                                std::process::exit(1);
                            };

                        let Ok(folders) = MultiSelect::new("Select folders to move to the new project", 
                            vec!["Desktop", "Downloads", "Documents", "Pictures", "Videos", "Music"])
                            .with_default(&[0,1,2,3,4,5])
                            .prompt() else {
                                std::process::exit(1);
                            };
                            
                        let folders: Vec<String> = folders
                            .into_iter()
                            .map(|folder| folder.to_owned())
                            .collect();

                        (parent, path, folders)
                    } else {
                        (
                            parent.as_ref().map(|s|s.clone()).unwrap_or(config.all_prjs.name.clone()),
                            path.as_ref().map(|s|s.clone()).unwrap_or(prj_name.clone()),
                            folders.clone().unwrap_or(vec!["Desktop".to_owned(), "Downloads".to_owned()])
                        )
                    };
    
                new_prj(&mut config, &prj_name, folders, path, parent)
                    .expect("Could not create new project");
            },
            SubCommands::Remove => {
                println!("Removing Project \"{prj_name}\", folders will not be deleted");
                
                let deleted = config.all_prjs.walk(&prj_name, &prj_name, |_,_| true,
                    |p, target,_,_| {
                    if let Some(index) =
                        p.children.iter()
                            .position(|pr| &pr.name == target) 
                    {
                        p.children.remove(index);
                        true
                    } else {
                        false
                    }
                });

                if deleted.is_some() && deleted.unwrap() {
                    println!("Sucessfully removed project");
                } else {
                    eprintln!("Didnt't find project to remove");
                }
    
                // if the active project is removed, set the active project to the root project
                if prj_name == config.active {
                    prj_name = config.all_prjs.name.clone();
                }
            },
            SubCommands::Rename {ref new_name, ref new_path} => {
                let (parent_path, old_path) = config.all_prjs.walk(&prj_name, &(new_name, new_path),
                    |p, new_data| {
                        let old_path = p.path.clone();
                        if let Some(new_name) = new_data.0.clone() {
                            p.name = new_name;
                        }
                        if let Some(new_path) = new_data.1.clone() {
                            p.path = new_path.clone();
                        }
                        (PathBuf::new(), old_path)
                    },
                    |p, _, (child_path, old_path), _| (PathBuf::from_iter(vec![PathBuf::from(&p.path), child_path]), old_path))
                    .expect("Could not find project path");
    
                if let Some(new_path) = new_path.as_ref() {
                    let mut to = parent_path.clone();
                    to.push(new_path);
    
                    let mut from = parent_path.clone();
                    from.push(old_path);
                    
                    if !from.exists() || !from.is_dir() {
                        eprintln!("Could not find project folder to rename");
                    }
                    fs::rename(&from, &to)
                        .expect("Could not rename");
                }
                if let Some(new_name) = new_name {
                    prj_name = new_name.clone();
                }
            },
            SubCommands::GetPath => {
                let url = config.all_prjs.walk(&prj_name, &(),
                    |p, _| PathBuf::from(&p.path),
                    |p, _, child_path, _| PathBuf::from_iter(vec![PathBuf::from(&p.path), child_path]))
                    .expect("Could not find project path");
                let Some(url) = url.to_str() else {
                    panic!("Could not convert path to string");
                };
                println!("{url}");
                return;
            },
            SubCommands::Init => (),
        }
    }
    
    //Change Project
    let after_scripts =
        match change_prj(&prj_name, &mut config, config_dir) {
            Ok(after_scripts) => {
                println!("Changed to Project {}", prj_name);
                Some(after_scripts)
            },
            Err(e) => {
                println!("Could not change to Project {}, Error: {}", prj_name, e);
                None
            }
        };

    //Write Config to json
    std::fs::write(&path,serde_json::to_string_pretty(&config).unwrap())
            .expect("Could not write initial config file");


    // Execute the Scripts after changing the project
    if let Some((after_scripts, env_vars)) = after_scripts {
        fn execute(script: PathBuf, env_vars: &HashMap<String, String>) {
            if script.is_file() {
                if let Ok(mut child) = std::process::Command::new("sh")
                    .envs(env_vars)
                    .arg("-c")
                    .arg(&script)
                    .spawn() {
                        let _ = child.wait();
                    }
            }
        }

        for script in after_scripts {
            execute(script, &env_vars);
        }
    }
}

impl Project {
    /**
        Walks through the project tree recursively and applies the function ```gen``` to the target project.
        ```res``` get called on the way back down the tree only on the branch that found the target.
    */
    fn walk<ResultT, DataT>(&mut self, target: &str, data: &DataT, gen: fn(&mut Project, &DataT) -> ResultT, res: fn(&mut Project, &DataT, ResultT, fn(&mut Project, &DataT) -> ResultT) -> ResultT) -> Option<ResultT> {
        if self.name == target {
            return Some(gen(self, data));
        }
        let child_data = self.children.iter_mut()
            .find_map(|pr| pr.walk(target, data, gen, res));

        // a child is part of the target branch
        if let Some(child_path) = child_data {
            return Some(res(self, data, child_path, gen));
        };
        None
    }
}