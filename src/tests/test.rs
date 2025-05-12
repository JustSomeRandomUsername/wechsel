use std::path::PathBuf;

use dirs::home_dir;
use rand::random_bool;

use crate::{
    PROJECT_EXTENSION, WECHSEL_FOLDER_EXTENSION,
    change::{CURRENT_PROJECT_FOLDER, get_enviroment_vars_fish_path, get_enviroment_vars_path},
    init::{DEFAULT_ROOT_PRJ, bashrc_path, fish_config_path, on_prj_change_path},
    tests::utils::{FindPrj, PATH_TO_WECHSEL_BINARY, print_command_output},
    tree::TreeOutput,
    utils::{HOME_FOLDERS, get_config_dir, get_home_folder_paths, path_from_iter},
};

use super::utils::{
    assert_included, assert_includes_nothing_other_then, call_as_user, create_user_data,
    query_folder,
};

pub(crate) struct Project {
    pub name: String,
    pub folders: Vec<PathBuf>,
    // This is a list of all wechsel folders this prj or any parent has, so all folders that could change if this project gets wechseled
    pub all_relevant_folders: Vec<PathBuf>,
    pub parent: Option<String>,
}
#[test]
fn test1() {
    let home_dir = home_dir().expect("could not find home dir");

    create_user_data(&home_dir);
    init_test();
    init_test();
}

#[test]
fn test2() {
    let home_dir = home_dir().expect("could not find home dir");

    create_user_data(&home_dir);
    let home_prj = init_test();
    let prj1 = new_test("prj1", &home_prj);
    change_test(&home_prj);
    change_test(&prj1);
}

fn init_test() -> Project {
    println!("Init");
    let home_dir = home_dir().expect("could not find home dir");

    let before = query_folder(&home_dir);

    let output = call_as_user(&[PATH_TO_WECHSEL_BINARY, "init", "-y"], &home_dir);
    print_command_output(output);

    let after = query_folder(&home_dir);

    let home_prj = path_from_iter([
        &home_dir,
        &PathBuf::from(DEFAULT_ROOT_PRJ).with_extension(PROJECT_EXTENSION),
    ]);
    let config_dir = get_config_dir().expect("Could not find config dir");
    assert_included(
        after.iter(),
        get_home_folder_paths().map(|(_, path)| path).chain([
            home_prj.clone(),
            config_dir.clone(),
            path_from_iter([&home_dir, &PathBuf::from(CURRENT_PROJECT_FOLDER)]),
            on_prj_change_path(&config_dir),
        ]),
    );

    assert_includes_nothing_other_then(
        after.difference(&before),
        get_home_folder_paths().map(|(_, path)| path).chain([
            home_prj,
            config_dir.clone(),
            path_from_iter([&home_dir, &PathBuf::from(CURRENT_PROJECT_FOLDER)]),
            fish_config_path(&config_dir),
            bashrc_path(),
            on_prj_change_path(&config_dir),
            get_enviroment_vars_fish_path(&config_dir),
            get_enviroment_vars_path(&config_dir),
            path_from_iter(["/root", ".cache"]),
        ]),
    );

    let tree = get_current_tree().unwrap();
    assert!(tree.tree.find(DEFAULT_ROOT_PRJ).is_some());
    assert!(tree.active == DEFAULT_ROOT_PRJ);
    Project {
        name: DEFAULT_ROOT_PRJ.to_string(),
        folders: get_home_folder_paths().map(|(_, p)| p).collect(),
        all_relevant_folders: get_home_folder_paths().map(|(_, p)| p).collect(),
        parent: None,
    }
}

fn new_test(name: &str, parent: &Project) -> Project {
    println!("new: {name}");

    let home_dir = home_dir().expect("could not find home dir");

    let before = query_folder(&home_dir);

    let folder_list: &[&str] = &HOME_FOLDERS
        .into_iter()
        .filter(|_| random_bool(0.8))
        .collect::<Vec<_>>();
    let output = call_as_user(
        &[
            PATH_TO_WECHSEL_BINARY,
            "new",
            name,
            "-p",
            parent.name.as_str(),
            "--folders",
            &folder_list.join(" "),
        ],
        &home_dir,
    );

    print_command_output(output);

    let after = query_folder(&home_dir);

    let root_prj_path = path_from_iter([&home_dir, &PathBuf::from(DEFAULT_ROOT_PRJ)])
        .with_extension(PROJECT_EXTENSION);

    let new_prj_path =
        path_from_iter([&root_prj_path, &PathBuf::from(name)]).with_extension(PROJECT_EXTENSION);

    println!(
        "{before:#?} {after:#?} {:#?}",
        after.difference(&before).collect::<Vec<_>>()
    );

    let folders: Vec<PathBuf> = folder_list
        .iter()
        .map(|name| {
            path_from_iter([&new_prj_path, &PathBuf::from(name)])
                .with_extension(WECHSEL_FOLDER_EXTENSION)
        })
        .collect();

    let new_prj = Project {
        name: name.to_string(),
        folders: folders.clone(),
        all_relevant_folders: folders
            .into_iter()
            .chain(parent.all_relevant_folders.iter().cloned())
            .collect(),
        parent: Some(parent.name.clone()),
    };

    let folders = new_prj.all_relevant_folders.iter().cloned().chain([
        new_prj_path.clone(),
        path_from_iter([&home_dir, &PathBuf::from(CURRENT_PROJECT_FOLDER)]),
    ]);
    assert_included(after.iter(), folders.clone());
    assert_includes_nothing_other_then(after.difference(&before), folders);

    let tree = get_current_tree().unwrap();
    assert!(tree.tree.find(name).is_some());
    assert!(tree.active == name);

    new_prj
}

pub(crate) fn change_test(prj: &Project) {
    println!("change1: {}", prj.name);
    let home_dir = home_dir().expect("could not find home dir");

    let before = query_folder(&home_dir);

    let output = call_as_user(
        &[PATH_TO_WECHSEL_BINARY, "change", prj.name.as_str()],
        &home_dir,
    );
    print_command_output(output);

    let after = query_folder(&home_dir);

    let tree = get_current_tree().unwrap();
    assert!(tree.active == prj.name);

    assert_included(
        after.iter(),
        prj.all_relevant_folders
            .clone()
            .into_iter()
            .chain([path_from_iter([
                &home_dir,
                &PathBuf::from(CURRENT_PROJECT_FOLDER),
            ])]),
    );
    let config_dir = get_config_dir().expect("Could not find config dir");
    assert_includes_nothing_other_then(
        after.difference(&before),
        prj.all_relevant_folders.clone().into_iter().chain([
            path_from_iter([&home_dir, &PathBuf::from(CURRENT_PROJECT_FOLDER)]),
            get_enviroment_vars_fish_path(&config_dir),
            get_enviroment_vars_path(&config_dir),
        ]),
    );

    println!("{:?}", after.difference(&before).collect::<Vec<_>>());
}

fn get_current_tree() -> Option<TreeOutput> {
    let output = call_as_user(
        &[PATH_TO_WECHSEL_BINARY, "tree"],
        &home_dir().expect("could not find home dir"),
    );

    serde_json::de::from_str::<TreeOutput>(
        String::from_utf8(output.stdout)
            .unwrap()
            .replace("\\n", "\n")
            .as_str(),
    )
    .ok()
}
