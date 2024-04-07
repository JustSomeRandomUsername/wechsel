use clap::{Parser, Subcommand};
use init::init_prj;
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
    command : ArgEnums,

    project_name: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum ArgEnums {
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
    Initial,
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
    GetPath
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
        // No Config File
        if matches!(args.command, ArgEnums::Initial) {
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
    let mut prj_name = prj_name.unwrap();
    
    match args.command {
        ArgEnums::New {ref parent, ref path, ref folders} => {
            let parent = parent.as_ref().map(|s|s.clone()).unwrap_or(config.all_prjs.name.clone());
            let path = path.as_ref().map(|s|s.clone()).unwrap_or(prj_name.clone());
            let folders = folders.clone().unwrap_or(vec!["Desktop".to_owned(), "Downloads".to_owned()]);

            new_prj(&args, &mut config, &prj_name, folders, path, parent)
                .expect("Could not create new project");
        },
        ArgEnums::Remove => {
            println!("Removing Project {}, folders will not be deleted", prj_name);
    
            config.all_prjs.walk(&prj_name, &prj_name, |_,_| {},
                |p, target,_,_|{
                let index = p.children.iter()
                    .position(|pr| &pr.name == target)
                    .expect("Could not find project to remove");
                p.children.remove(index);
            });

            // if the active project is removed, set the active project to the root project
            if prj_name == config.active {
                prj_name = config.all_prjs.name.clone();
            }
        },
        ArgEnums::Rename {ref new_name, ref new_path} => {
            let (parent_path, target_path) = config.all_prjs.walk(&prj_name, &(new_name, new_path),
                |p, new_data| {
                    if let Some(new_name) = new_data.0.clone() {
                        p.name = new_name;
                    }
                    if let Some(new_path) = new_data.1.clone() {
                        p.path = new_path.clone();
                    }
                    (PathBuf::new(), p.path.clone())
                },
                |p, _, (child_path, target_path), _| (PathBuf::from_iter(vec![PathBuf::from(&p.path), child_path]), target_path))
                .expect("Could not find project path");

            if let Some(new_path) = new_path.as_ref() {
                let mut to = parent_path.clone();
                to.push(new_path);

                let mut from = parent_path;
                from.push(target_path);

                fs::rename(from, to)
                    .expect("Could not rename");
            }
            if let Some(new_name) = new_name {
                prj_name = new_name.clone();
            }
        },
        ArgEnums::GetPath => {
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
        ArgEnums::Initial => (),
    }
    // if args.initial {
    //     //Dont jumpt into new, remove ..

    // } else if args.new {
    //     //New Project
    //     if let Some(name) = prj_name.as_ref() {
    //         new_prj(&args, &mut config, name, folders).expect("Could not create new project");
    //     }
    // } else if args.remove {
    //     //Remove Project
    //     if let Some(name) = prj_name.as_ref() {    
    //         println!("Removing Project {}, folders will not be deleted", name);
    
    //         config.all_prjs.walk(name, name, |_,_| {},
    //             |p, target,_,_|{
    //             let index = p.children.iter()
    //                 .position(|pr| &pr.name == target)
    //                 .expect("Could not find project to remove");
    //             p.children.remove(index);
    //         });
    //     }
    //     if prj_name.is_some() && prj_name.as_ref().unwrap() == &config.active {
    //         prj_name = Some(config.all_prjs.name.clone());   
    //     }
    // } else if args.rename {

    //     let new_name = args.new_name.as_ref().unwrap_or(args.project_name.as_ref().unwrap());
    //     let from = config.all_prjs.walk(prj_name.as_ref().unwrap(), &(new_name, args.new_path.as_ref()),
    //             |p, new_data| {
    //                 let old_path = PathBuf::from(&p.path);
    //                 p.name = new_data.0.clone();
    //                 if let Some(new_path) = new_data.1 {
    //                     p.path = new_path.clone();
    //                 }
    //                 old_path
    //             },
    //             |p, _, child_path, _| PathBuf::from_iter(vec![PathBuf::from(&p.path), child_path]))
    //             .expect("Could not find project path");

    //         if let Some(new_path) = args.new_path.as_ref() {
    //             let mut to = from.clone();
    //             to.pop();
    //             to.push(new_path);
    //             fs::rename(from, to)
    //                 .expect("Could not rename");
    //         }

    //         prj_name = Some(new_name.clone());
    // } else if args.get_url {
    //     if let Some(name) = prj_name.as_ref() {
    //         let url = config.all_prjs.walk(&name, &(),
    //             |p, _| p.path.clone(), 
    //             |p, _, child_path, _| format!("{}/{}", p.path, child_path))
    //             .expect("Could not find project path");
    //         println!("{}", url);
    //     }
    //     return;
    // }

    //Change Project
    let after_scripts =
        match change_prj(&prj_name, &mut config) {
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
            println!("Executing script {:?}", env_vars);
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
            .map(|pr| pr.walk(target, data, gen, res))
            .find(|op| op.is_some());

        if let Some(child_path) = child_data.flatten() {
            return Some(res(self, data, child_path, gen));
        };
        None
    }
}