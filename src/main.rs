use crate::change::change_prj;
use clap::{Parser, Subcommand};
use init::init_prj;
use inquire::{MultiSelect, Text};
use migrate::migrate_to_new_config;
use new::new_prj;
use std::{fs, vec};
use tree::{TreeOutput, get_project_tree};
use utils::{HOME_FOLDERS, get_config_dir, query_active_project};

mod change;
mod init;
mod migrate;
mod new;
mod tree;
mod utils;

#[cfg(test)]
mod tests;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    command: Option<Command>,

    project_name: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[clap(about = "[Default] Change the active project.")]
    Change {
        /// project to change to
        project_name: String,
    },
    #[clap(about = "Turn the working directory into a new project.")]
    New {
        /// Name of the new project
        project_name: String,

        /// parent project
        #[clap(short, long)]
        parent: Option<String>,
        /// folders to create in the new project
        #[clap(long, value_parser, num_args = 1.., value_delimiter = ' ')]
        folders: Option<Vec<String>>,
    },
    #[clap(
        about = "Initialize the config file and create a default project and move the folders to it."
    )]
    Init {
        #[clap(short, long)]
        yes: bool,
    },

    #[clap(about = "Returns the project tree structure as a json string")]
    Tree,
    #[clap(about = "Migrate old Json config based wechsel setup to the new folder based one")]
    Migrate {
        #[clap(short, long)]
        yes: bool,
    },
}

const PROJECT_EXTENSION: &str = "p";
const WECHSEL_FOLDER_EXTENSION: &str = "w";

pub fn main_with_args(args: Args) {
    let config_dir = get_config_dir().expect("No config folder found");
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).expect("Could not create config folder");
    }

    // Check if Init is selected
    let mut prj_name = if let Some(Command::Init { yes }) = args.command {
        Some(init_prj(config_dir.clone(), yes))
    } else {
        None
    };

    if prj_name.is_none() && args.project_name.is_some() {
        prj_name = args.project_name;
    }

    if let Some(cmd) = &args.command {
        match cmd {
            Command::New {
                project_name,
                parent,
                folders,
            } => {
                prj_name = Some(project_name.clone());

                let pwd = std::env::current_dir().expect("Could not get current dir");

                let found_parent = parent
                    .is_none()
                    .then(|| {
                        pwd.extension()
                            .map(|ext| ext == PROJECT_EXTENSION)
                            .unwrap_or_default()
                            .then(|| {
                                // current dir is a project
                                pwd.file_name()
                                    .and_then(|name| name.to_str())
                                    .expect("Could not get parent folder name")
                                    .to_string()
                            })
                    })
                    .flatten();

                let (parent, folders) = if parent.is_none() && folders.is_none() {
                    // If no options are set, ask for them interactively

                    let pwd = std::env::current_dir().expect("Could not get current dir");
                    let mut parent_folder = pwd.clone();
                    parent_folder.pop();

                    let parent = found_parent.unwrap_or_else(|| {
                        // not in a wechsel project so the user has to supply a parent
                        let Ok(parent) = Text::new("parent project")
                            .with_default(&query_active_project().unwrap_or_default())
                            .prompt()
                        else {
                            std::process::exit(1);
                        };
                        parent
                    });

                    let Ok(folders) = MultiSelect::new(
                        "Select folders to move to the new project",
                        HOME_FOLDERS.to_vec(),
                    )
                    .with_default(&[0, 1, 2, 3, 4, 5])
                    .prompt() else {
                        std::process::exit(1);
                    };

                    let folders: Vec<String> = folders
                        .into_iter()
                        .map(|folder| folder.to_owned())
                        .collect();

                    (parent, folders)
                } else {
                    (
                        parent
                            .clone()
                            .unwrap_or(query_active_project().unwrap_or_default()),
                        folders
                            .clone()
                            .unwrap_or(vec!["Desktop".to_owned(), "Downloads".to_owned()]),
                    )
                };

                new_prj(project_name, folders, parent, &config_dir)
                    .expect("Could not create new project");
            }
            Command::Change { project_name } => prj_name = Some(project_name.clone()),
            Command::Init { .. } => (),
            Command::Tree => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&TreeOutput {
                        tree: get_project_tree(),
                        active: query_active_project().unwrap_or_default(),
                    })
                    .unwrap_or_default()
                )
            }
            Command::Migrate { yes } => {
                migrate_to_new_config(&config_dir, *yes);
            }
        }
    }

    if let Some(prj_name) = prj_name {
        //Change Project
        match change_prj(&prj_name, config_dir) {
            Ok(_) => {
                println!("Changed to Project {}", prj_name);
            }
            Err(e) => {
                println!("Could not change to Project {}, Error: {}", prj_name, e);
            }
        }
    }
}
fn main() {
    let args = Args::parse();

    main_with_args(args);
}
