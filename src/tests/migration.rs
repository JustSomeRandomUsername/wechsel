use std::{
    fs,
    path::{Path, PathBuf},
};

use dirs::home_dir;
use rand::{Rng, distr::Alphanumeric};

use crate::{
    init::DEFAULT_ROOT_PRJ,
    migrate::{OldConfig, OldProject, get_old_config_file_path},
    tests::{
        test::Project,
        utils::{PATH_TO_WECHSEL_BINARY, call_as_user, print_command_output},
    },
    utils::{HOME_FOLDERS, get_config_dir, get_home_folder_paths, path_from_iter},
};

use super::{
    test::change_test,
    utils::{create_user_data, generate_files},
};

pub const PROJECTS_FOLDER: &str = "projects";
#[test]
fn migration_test() {
    let home_dir = home_dir().expect("could not find home dir");
    create_user_data(&home_dir);

    let old_config = setup_migration(
        &home_dir,
        &get_config_dir().expect("could not find config dir"),
    );

    let prjs = perform_migration(&home_dir, old_config);
    for prj in prjs {
        change_test(&prj)
    }
}

pub(crate) fn perform_migration(home_dir: &PathBuf, old_config: OldConfig) -> Vec<Project> {
    let output = call_as_user(&[PATH_TO_WECHSEL_BINARY, "migrate"], home_dir);

    print_command_output(output);

    let mut projects = vec![];

    fn convert_config(prj: OldProject, parent: Option<&Project>, projects: &mut Vec<Project>) {
        let mut home_folders = get_home_folder_paths();
        let folders: Vec<PathBuf> = prj
            .folder
            .iter()
            .filter_map(|folder| {
                home_folders
                    .find(|(name, _)| name == folder)
                    .map(|(_, path)| path)
            })
            .collect();
        let new_prj = Project {
            name: prj.name,
            folders: folders.clone(),
            all_relevant_folders: folders
                .into_iter()
                .chain(
                    parent
                        .as_ref()
                        .map(|parent| parent.folders.clone().into_iter())
                        .unwrap_or(vec![].into_iter()),
                )
                .collect(),
            parent: parent.map(|par| par.name.clone()),
        };
        for child in prj.children {
            convert_config(child, Some(&new_prj), projects);
        }
        projects.push(new_prj);
    }
    convert_config(old_config.all_prjs, None, &mut projects);
    projects
}

pub(crate) fn setup_migration(home_dir: &PathBuf, config_dir: &Path) -> OldConfig {
    let config_path = get_old_config_file_path(config_dir);

    let conf = OldConfig {
        active: DEFAULT_ROOT_PRJ.to_string(),
        all_prjs: OldProject {
            name: DEFAULT_ROOT_PRJ.to_string(),
            path: path_from_iter([home_dir, &PathBuf::from(PROJECTS_FOLDER)])
                .to_str()
                .unwrap()
                .to_string(),
            folder: vec![
                "default/Desktop".to_string(),
                "default/Downloads".to_string(),
                "default/Documents".to_string(),
                "default/Pictures".to_string(),
                "default/Videos".to_string(),
                "default/Music".to_string(),
            ],
            children: vec![
                generate_rand_prj(vec![]),
                generate_rand_prj(vec![generate_rand_prj(vec![])]),
            ],
        },
    };

    generate_prj_files(&conf.all_prjs, None);

    fs::create_dir(config_dir).unwrap();
    serde_json::to_string(&conf)
        .ok()
        .and_then(|content| fs::write(config_path, content).ok())
        .expect("Could not open old config file");
    conf
}

fn generate_prj_files(prj: &OldProject, parent_path: Option<&PathBuf>) {
    let path = parent_path
        .map(|parent| path_from_iter([parent, &PathBuf::from(&prj.path)]))
        .unwrap_or(PathBuf::from(&prj.path));

    for folder in prj
        .folder
        .iter()
        .map(|fold| path_from_iter([&path, &PathBuf::from(fold)]))
    {
        fs::create_dir_all(&folder).unwrap();
        generate_files(&folder, 0);
    }
    generate_files(&path, 1);

    for child in prj.children.iter() {
        generate_prj_files(child, Some(&path));
    }
}

fn generate_rand_prj(children: Vec<OldProject>) -> OldProject {
    let name: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(4)
        .map(char::from)
        .collect();
    OldProject {
        name: name.clone(),
        path: name,
        folder: HOME_FOLDERS.into_iter().map(|a| a.to_string()).collect(),
        children,
    }
}
