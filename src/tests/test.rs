use std::{
    fs,
    os::unix::fs::{MetadataExt, symlink},
    path::PathBuf,
};

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
    assert_included, assert_includes_nothing_other_then, assert_prj_on_change_test, call_as_user,
    query_folder, setup_home, setup_on_change_test,
};

pub(crate) struct Project {
    pub name: String,
    pub path: PathBuf,
    // These are the paths to the folders in the home dir that this project has wechsel folders for
    pub folders: Vec<PathBuf>,
    // This is a list of all wechsel folders this prj or any parent has, so all folders that could change if this project gets wechseled
    pub all_relevant_folders: Vec<PathBuf>,
    pub parent: Option<String>,
}
#[test]
fn test1() {
    let home_dir = home_dir().expect("could not find home dir");

    setup_home(&home_dir, true);
    init_test();
    init_test();
}

#[test]
fn test2() {
    let home_dir = home_dir().expect("could not find home dir");

    setup_home(&home_dir, true);
    assert!(
        get_current_tree().is_none(),
        "Wechsel tree should error if not initialized"
    );
    let home_prj = init_test();
    let prj1 = new_test("prj1", &home_prj);

    setup_on_change_test(&prj1.path);
    change_test(&prj1);
    assert_prj_on_change_test(&prj1);

    setup_on_change_test(&home_prj.path);
    change_test(&home_prj);
    assert_prj_on_change_test(&home_prj);

    let new_destination = path_from_iter([&home_dir, &PathBuf::from("test_prj")]);
    let old_destination = &prj1.path;
    fs::rename(old_destination, &new_destination).unwrap();
    symlink(new_destination, old_destination).unwrap();

    let new_destination = path_from_iter([&home_dir, &PathBuf::from("test_wechsel_folder")]);
    let old_destination = path_from_iter([
        &prj1.path,
        &PathBuf::from(prj1.folders[0].file_name().unwrap()),
    ])
    .with_extension(WECHSEL_FOLDER_EXTENSION);
    fs::rename(&old_destination, &new_destination).unwrap();
    symlink(new_destination, &old_destination).unwrap();

    change_test(&prj1);
    assert_prj_on_change_test(&prj1);

    let tree = get_current_tree().unwrap();
    assert!(tree.tree.find(&prj1.name).is_some());
}

pub(crate) fn init_test() -> Project {
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
            home_prj.clone(),
            config_dir.clone(),
            path_from_iter([&home_dir, &PathBuf::from(CURRENT_PROJECT_FOLDER)]),
            fish_config_path(&config_dir),
            bashrc_path(),
            on_prj_change_path(&config_dir),
            get_enviroment_vars_fish_path(&config_dir),
            get_enviroment_vars_path(&config_dir),
            path_from_iter(["/root", ".cache"]),
        ]),
        "init",
    );

    let tree = get_current_tree().unwrap();
    assert!(tree.tree.find(DEFAULT_ROOT_PRJ).is_some());
    assert!(tree.active == DEFAULT_ROOT_PRJ);
    Project {
        path: home_prj,
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

    let folders: Vec<PathBuf> = folder_list
        .iter()
        .map(|name| path_from_iter([&home_dir, &PathBuf::from(name)]))
        .collect();

    let new_prj = Project {
        name: name.to_string(),
        path: new_prj_path.clone(),
        folders: folders.clone(),
        all_relevant_folders: folders
            .into_iter()
            .chain(parent.all_relevant_folders.iter().cloned())
            .collect(),
        parent: Some(parent.name.clone()),
    };

    let folders = new_prj
        .all_relevant_folders
        .iter()
        .cloned()
        .chain([
            new_prj_path.clone(),
            path_from_iter([&home_dir, &PathBuf::from(CURRENT_PROJECT_FOLDER)]),
        ])
        .chain(folder_list.iter().map(|name| {
            path_from_iter([&new_prj_path, &PathBuf::from(name)])
                .with_extension(WECHSEL_FOLDER_EXTENSION)
        }));
    assert_included(after.iter(), folders.clone());
    assert_includes_nothing_other_then(after.difference(&before), folders, "new");

    let tree = get_current_tree().unwrap();
    assert!(tree.tree.find(name).is_some());
    assert!(tree.active == name);

    new_prj
}

pub(crate) fn change_test(prj: &Project) {
    println!("change1: {}", prj.name);
    let home_dir = home_dir().expect("could not find home dir");
    let config_dir = get_config_dir().expect("Could not find config dir");

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
    assert_includes_nothing_other_then(
        after.difference(&before),
        prj.all_relevant_folders.clone().into_iter().chain([
            path_from_iter([&home_dir, &PathBuf::from(CURRENT_PROJECT_FOLDER)]),
            get_enviroment_vars_fish_path(&config_dir),
            get_enviroment_vars_path(&config_dir),
        ]),
        "change",
    );

    // Assert that the wechsel folders of the target project got symlinked correctly
    for fold in prj.folders.iter() {
        if let Ok(home_meta) = fold.metadata() {
            let folder_target =
                path_from_iter([&prj.path, &PathBuf::from(fold.file_name().unwrap())])
                    .with_extension(WECHSEL_FOLDER_EXTENSION);
            if let Ok(target_meta) = folder_target.metadata() {
                assert!(
                    home_meta.dev() == target_meta.dev() && home_meta.ino() == target_meta.ino()
                )
            } else {
                panic!("Err 12 {folder_target:?}")
            }
        } else {
            panic!("Err 122")
        }
    }
    //Assert that ~/Project is symlinked correctly
    assert!(
        path_from_iter([&home_dir, &PathBuf::from(CURRENT_PROJECT_FOLDER)])
            .metadata()
            .and_then(|meta| prj.path.metadata().map(|target_meta| (meta, target_meta)))
            .map(|(m1, m2)| m1.dev() == m2.dev() && m1.ino() == m2.ino())
            .unwrap_or(false),
        "~/{CURRENT_PROJECT_FOLDER} is not symlinked correctly to {:?}",
        &prj.path
    );
}

pub(crate) fn get_current_tree() -> Option<TreeOutput> {
    let output = call_as_user(
        &[PATH_TO_WECHSEL_BINARY, "tree"],
        &home_dir().expect("could not find home dir"),
    );
    if !output.status.success() && !output.stderr.is_empty() {
        println!("{}", String::from_utf8(output.stderr).unwrap());
        return None;
    }
    serde_json::de::from_str::<TreeOutput>(
        String::from_utf8(output.stdout)
            .unwrap()
            .replace("\\n", "\n")
            .as_str(),
    )
    .ok()
}
