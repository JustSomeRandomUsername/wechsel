use std::{
    collections::BTreeSet,
    fmt::Debug,
    fs::{self, Metadata},
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::PathBuf,
    process::{Command, Output},
};

use rand::{Rng, distr::Alphanumeric, random};
use walkdir::WalkDir;
use wechsel::{
    CURRENT_PROJECT_FOLDER, HOME_FOLDERS, ProjectTreeNode, get_config_dir, get_home_folder_paths,
    path_from_iter,
};

use super::Project;

pub const PATH_TO_WECHSEL_BINARY: &str = "wechsel";
pub const PROJECT_ON_CHANGE_FILE_NAME: &str = ".on-prj-change";
pub const ON_CHANGE_OUTPUT_FILE: &str = "/tmp/test";
pub const ON_CHANGE_TEST_SCRIPT: &str = "echo $PRJ > ";
pub struct File(PathBuf, Metadata);

impl Debug for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.1.dev() == other.1.dev() && self.1.ino() == other.1.ino()
    }
}
impl Eq for File {}
impl PartialOrd for File {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.1.ino().cmp(&other.1.ino()))
    }
}
impl Ord for File {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.1.ino().cmp(&other.1.ino())
    }
}
pub fn assert_includes_nothing_other_then<
    'a,
    I2: Iterator<Item = &'a File>,
    I: IntoIterator<Item = PathBuf>,
>(
    diff: I2,
    other: I,
    test_name: &str,
) {
    let diff = diff.map(|file| &file.0).collect::<Vec<_>>();
    let other = other.into_iter().collect::<Vec<_>>();
    for item in diff {
        assert!(
            other.contains(item),
            "{item:?} was not allowed in {test_name}-Test",
        );
    }
}
pub fn assert_included<'a, I2: Iterator<Item = &'a File>, I: IntoIterator<Item = PathBuf>>(
    diff: I2,
    other: I,
) {
    let diff = diff.map(|file| &file.0).collect::<Vec<_>>();

    for item in other {
        assert!(diff.contains(&&item), "assert: didn't find {item:?}");
    }
}

pub fn assert_included_symlinks<
    'a,
    I2: Iterator<Item = &'a File>,
    I: IntoIterator<Item = PathBuf>,
>(
    diff: I2,
    other: I,
) {
    let (paths, metadata): (Vec<_>, Vec<_>) = diff.map(|file| (&file.0, &file.1)).unzip();

    for item in other {
        let idx = paths.iter().position(|path| path == &&item);
        assert!(idx.is_some(), "assert: didn't find {item:?}");
        assert!(
            metadata.get(idx.unwrap()).unwrap().is_symlink(),
            "{item:?} is not a symlink"
        );
    }
}

pub fn call_as_user<'a, T: Clone + IntoIterator<Item = &'a &'a str>>(
    cmd: T,
    dir: &PathBuf,
) -> Output {
    let mut iter = cmd.into_iter();
    Command::new(iter.next().unwrap())
        .args(iter)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute command {e:?}"))
}
pub fn print_command_output(output: Output) {
    println!(
        "{}",
        String::from_utf8(output.stdout)
            .unwrap()
            .replace("\\n", "\n")
    );
    println!(
        "{}",
        String::from_utf8(output.stderr)
            .unwrap()
            .replace("\\n", "\n")
    );
}
pub fn setup_home(home: &PathBuf, create_folders: bool) {
    let folders = HOME_FOLDERS;

    println!("Setting up test enviromnent");
    if get_home_folder_paths().count() < 6 {
        eprintln!(
            "Didnt find paths for all 6 excpected home directories, please check that the xdg-user-dir enviroment variables are set, calling `xdg-user-dirs-update` might do so",
        )
    }
    println!("Deleting previous files");
    call_as_user(
        [
            "rm",
            "-r",
            get_config_dir()
                .expect("Could not find config dir")
                .as_os_str()
                .to_str()
                .unwrap(),
        ]
        .iter(),
        home,
    );
    call_as_user(["bash", "-c", "rm -r *p"].iter(), home);

    call_as_user(
        ["rm", "-r", CURRENT_PROJECT_FOLDER, "test_prj"]
            .iter()
            .chain(folders.iter()),
        // .chain([PROJECTS_FOLDER].iter()),TODO only needed for migrations
        home,
    );

    if create_folders {
        println!("Creating User Folders");
        call_as_user(["mkdir"].iter().chain(folders.iter()), home);
        for folder in folders {
            generate_files(&path_from_iter([home, &PathBuf::from(folder)]), 0);
        }
    }
    println!("Setup Done");
}

pub(crate) fn setup_on_change_test(path: &PathBuf) {
    let on_prj_change = path_from_iter([path, &PathBuf::from(PROJECT_ON_CHANGE_FILE_NAME)]);
    fs::write(
        &on_prj_change,
        format!("{ON_CHANGE_TEST_SCRIPT}{ON_CHANGE_OUTPUT_FILE}"),
    )
    .unwrap();
    let mut permissions = fs::metadata(&on_prj_change)
        .expect("Could not get metadata")
        .permissions();

    // Add execute permission
    permissions.set_mode(permissions.mode() | 0b001001001);

    fs::set_permissions(&on_prj_change, permissions).expect("Could not set permissions");
}
pub(crate) fn assert_prj_on_change_test(prj: &Project) {
    let test_name = fs::read(ON_CHANGE_OUTPUT_FILE)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap();

    assert!(
        test_name.trim() == prj.name.trim(),
        "on_change test failed: {} != {}",
        test_name.trim(),
        prj.name.trim()
    );
}
pub fn generate_files(dir: &PathBuf, depth: usize) {
    for _ in 0..(random::<f32>() * 4.0) as usize {
        let name: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(4)
            .map(char::from)
            .collect();

        let _ = fs::write(path_from_iter([dir, &PathBuf::from(&name)]), &name);

        if rand::random_bool(1.0 / (depth as f64 + 1.0)) {
            let folder = format!("{name}_dir");
            call_as_user(&["mkdir", folder.as_str()], dir);
            generate_files(&path_from_iter([dir, &PathBuf::from(folder)]), depth + 1);
        }
    }
}

pub fn query_folder(home: &PathBuf) -> BTreeSet<File> {
    WalkDir::new(home)
        .into_iter()
        .filter_map(Result::ok)
        .map(|e| {
            File(
                PathBuf::from(e.path()),
                e.metadata().expect("Could not get metadata"),
            )
        })
        .collect()
}

pub fn security_check(/*home: &PathBuf*/) {
    let output = Command::new("id")
        .arg("-u")
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute id -u. {e:?}"));
    if String::from_utf8(output.stdout)
        .unwrap()
        .replace("\\n", "\n")
        .trim()
        != "0"
    {
        eprintln!(
            "These tests should run in a container environment, they will delete and create files in the user directory."
        );
        std::process::exit(1);
    }

    // if read_dir(home)
    //     .map(|mut dirs| dirs.next().is_some())
    //     .unwrap_or_default()
    // {
    //     eprintln!(
    //         "The users home directory should be empty at the start of the test;"
    //     );
    //     std::process::exit(1);
    // }
}
pub trait FindPrj {
    fn find(self, prj_name: &str) -> Option<ProjectTreeNode>;
}

impl FindPrj for ProjectTreeNode {
    fn find(self, prj_name: &str) -> Option<ProjectTreeNode> {
        if self.prj_name == prj_name {
            return Some(self);
        }
        for child in self.children {
            if let Some(found) = child.find(prj_name) {
                return Some(found);
            }
        }
        None
    }
}
