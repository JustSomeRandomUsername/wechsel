use crate::{change::change_prj, new::new_prj_cmd};
use clap::{Parser, Subcommand};
use init::init_prj;
use migrate::migrate_to_new_config;
use std::fs;
use tree::{TreeOutput, get_project_tree};
use utils::{get_config_dir, query_active_project};

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

        #[clap(short, long)]
        parent: Option<String>,
        #[clap(long, value_parser, num_args = 1.., value_delimiter = ' ', help="the list of folders to create in the new project")]
        folders: Option<Vec<String>>,
    },
    #[clap(
        about = "Initialize the config file and create a default project and move the folders to it."
    )]
    Init {
        #[clap(short, long, help = "run non interactively with default values")]
        yes: bool,
    },

    #[clap(about = "Returns the project tree structure as a json string")]
    Tree {
        #[clap(long, help = "return the list of wechsel folders per project")]
        folders: bool,
    },
    #[clap(about = "Migrate old Json config based wechsel setup to the new folder based one")]
    Migrate {
        #[clap(short, long, help = "run non interactively with default values")]
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

    if args.project_name.is_none() && args.command.is_none() {
        eprintln!(
            "Either specify a command or a target project. Call wechsel --help for more information"
        );
        std::process::exit(1);
    }

    let mut prj_name = args.project_name;

    if let Some(cmd) = args.command {
        match cmd {
            Command::New {
                project_name,
                parent,
                folders,
            } => {
                prj_name = Some(project_name.clone());
                new_prj_cmd(parent, folders, &project_name, &config_dir);
            }
            Command::Change { project_name } => prj_name = Some(project_name.clone()),
            Command::Tree { folders } => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&TreeOutput {
                        tree: get_project_tree(&config_dir, folders),
                        active: query_active_project().unwrap_or_default(),
                    })
                    .unwrap_or_default()
                )
            }
            Command::Migrate { yes } => {
                migrate_to_new_config(&config_dir, yes);
            }
            Command::Init { yes } => prj_name = Some(init_prj(config_dir.clone(), yes)),
        }
    }

    if let Some(prj_name) = prj_name {
        //Change Project
        match change_prj(&prj_name, config_dir) {
            Ok(_) => {
                println!("Changed to Project {prj_name}");
            }
            Err(e) => {
                println!("Could not change to Project {prj_name}, Error: {e}");
            }
        }
    }
}
fn main() {
    let args = Args::parse();

    main_with_args(args);
}
