use std::{
    fs,
    path::{Path, PathBuf},
};

use dirs::home_dir;
use rand::{Rng, distr::Alphanumeric};
use wechsel::{
    DEFAULT_ROOT_PRJ, HOME_FOLDERS, PROJECT_EXTENSION, get_config_dir, get_home_folder_paths,
    path_from_iter,
};

// use wechsel::{
//     DEFAULT_ROOT_PRJ,
//     OldConfig,
//     OldProject,
//     PATH_TO_WECHSEL_BINARY,
//     PROJECT_EXTENSION,
//     assert_prj_on_change_test,
//     call_as_user,
//     get_old_config_file_path_unchecked,
//     print_command_output,
//     security_check,
//     // test::{Project, get_current_tree},
//     utils::{HOME_FOLDERS, get_config_dir, get_home_folder_paths, path_from_iter},
// };

// use super::{
//     test::{change_test, init_test},
//     utils::{generate_files, setup_home, setup_on_change_test},
// };

use crate::{Project, change_test, get_current_tree, init_test, utils::*};

pub const PROJECTS_FOLDER: &str = "projects";

#[test]
fn migration_test() {
    security_check();
    let home_dir = home_dir().expect("could not find home dir");
    setup_home(&home_dir, false);

    let old_config = setup_migration(
        &home_dir,
        &get_config_dir().expect("could not find config dir"),
    );
    assert!(
        get_current_tree().is_none(),
        "Wechsel tree should error if an old setup is present"
    );

    let prjs = perform_migration(&home_dir, old_config);
    for prj in prjs.iter() {
        change_test(prj);
    }
    init_test();
    for prj in prjs.iter() {
        change_test(prj);
        assert_prj_on_change_test(prj);
    }
}

pub(crate) fn perform_migration(home_dir: &PathBuf, old_config: OldConfig) -> Vec<Project> {
    let output = call_as_user(&[PATH_TO_WECHSEL_BINARY, "migrate", "-y"], home_dir);

    print_command_output(output);

    let mut projects = vec![];

    fn convert_config(
        prj: OldProject,
        parent: Option<&Project>,
        projects: &mut Vec<Project>,
        home_dir: &PathBuf,
    ) {
        let folders: Vec<PathBuf> = prj
            .folder
            .iter()
            .filter_map(|folder| {
                get_home_folder_paths()
                    .find(|(name, _)| {
                        name == &PathBuf::from(folder).file_name().unwrap().to_str().unwrap()
                    })
                    .map(|(_, path)| path)
            })
            .collect();

        let new_prj = Project {
            name: prj.name.clone(),
            path: path_from_iter([
                parent
                    .map(|parent| parent.path.clone())
                    .as_ref()
                    .unwrap_or(home_dir),
                &PathBuf::from(prj.name),
            ])
            .with_extension(PROJECT_EXTENSION),
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
        };
        for child in prj.children {
            convert_config(child, Some(&new_prj), projects, home_dir);
        }
        projects.push(new_prj);
    }
    convert_config(old_config.all_prjs, None, &mut projects, home_dir);
    projects
}

pub(crate) fn setup_migration(home_dir: &PathBuf, config_dir: &Path) -> OldConfig {
    let config_path = get_old_config_file_path_unchecked(config_dir);

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

    setup_on_change_test(&path);

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
