use std::{
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

#[cfg(test)]
use serde::Deserialize;
use serde::Serialize;

use crate::{
    PROJECT_EXTENSION, WECHSEL_FOLDER_EXTENSION,
    utils::{is_entry_folder_with_extension, path_from_iter},
};

#[derive(Serialize)]
#[cfg_attr(test, derive(Deserialize, Debug))]
pub struct TreeOutput {
    pub tree: ProjectTreeNode,
    pub active: String,
}
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize))]
pub struct ProjectTreeNode {
    #[serde(rename(serialize = "name", deserialize = "name"))]
    pub prj_name: String,
    pub children: Vec<ProjectTreeNode>,
    pub path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folders: Option<Vec<String>>,
}

pub struct FoundProject {
    pub parent: Option<Rc<FoundProject>>,
    pub path: PathBuf,
}

fn recursion_fn<
    Out,
    Parent,
    F: Fn(String, Vec<Out>, PathBuf, Rc<Parent>, Vec<String>) -> Out,
    F2: Fn(&String, &PathBuf, Option<Rc<Parent>>) -> Parent,
>(
    lambda: F,
    lambda_parent: F2,
    config_dir: &PathBuf,
    collect_folders: bool,
) -> Out {
    let home = dirs::home_dir().expect("Could not find home directory");

    fn inner<
        Out,
        Parent,
        F: Fn(String, Vec<Out>, PathBuf, Rc<Parent>, Vec<String>) -> Out,
        F2: Fn(&String, &PathBuf, Option<Rc<Parent>>) -> Parent,
    >(
        path: PathBuf,
        depth: usize,
        lambda: &F,
        parent: Option<Rc<Parent>>,
        lambda_parent: &F2,
        config_dir: &PathBuf,
        collect_folders: bool,
    ) -> Out {
        let prj_name = if depth == 0 {
            "home".to_string()
        } else {
            path.file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string()
        };

        let parent_out = Rc::new(lambda_parent(&prj_name, &path, parent));

        let mut folders = vec![];

        // is set to true after depth 0 to stop all the checks that are not needed after depth 0
        let mut has_wechsel_folder = depth != 0;

        let mut children: Vec<_> =
            fs::read_dir(&path)
                .map(|children| {
                    children
                        .into_iter()
                        .filter_map(|child| {
                            if collect_folders {
                                if is_entry_folder_with_extension(&child, WECHSEL_FOLDER_EXTENSION)
                                    .is_some()
                                {
                                    if let Some(folder) =
                                        child.as_ref().unwrap().path().file_stem().and_then(
                                            |stem| stem.to_str().map(|str| str.to_string()),
                                        )
                                    {
                                        folders.push(folder);
                                    }
                                    has_wechsel_folder = true;
                                }
                            } else if !has_wechsel_folder
                                && is_entry_folder_with_extension(&child, WECHSEL_FOLDER_EXTENSION)
                                    .is_some()
                            {
                                has_wechsel_folder = true;
                            }

                            is_entry_folder_with_extension(&child, PROJECT_EXTENSION).map(|child| {
                                inner(
                                    child.path().clone(),
                                    depth + 1,
                                    lambda,
                                    Some(parent_out.clone()),
                                    lambda_parent,
                                    config_dir,
                                    collect_folders,
                                )
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

        if !has_wechsel_folder && children.len() == 1 {
            // Extra rule for depth 0, if there is only one child project and no wechsel folders take it out of the tree
            children.remove(0)
        } else if !has_wechsel_folder
            && children.is_empty()
            && get_old_config_file_path(config_dir).is_some()
        {
            eprintln!(
                "Your wechsel setup seems to be setup for an old version of wechsel, please migrate to the new wechsel setup, to do this you might want to downgrade wechsel to version <= 0.2.3 and call wechsel migrate"
            );
            std::process::exit(1);
        } else if !has_wechsel_folder && children.is_empty() {
            eprintln!(
                "Wechsel could not find any projects in your home directory, are you sure you initialized wechsel already? (wechsel init)"
            );
            std::process::exit(1);
        } else {
            lambda(prj_name, children, path, parent_out, folders)
        }
    }
    inner(
        home,
        0,
        &lambda,
        None,
        &lambda_parent,
        config_dir,
        collect_folders,
    )
}

pub fn search_for_projects<const N: usize>(
    targets: [&str; N],
    config_dir: &PathBuf,
) -> [Option<Rc<FoundProject>>; N] {
    recursion_fn(
        |name, children: Vec<[Option<Rc<FoundProject>>; N]>, _, me, _| {
            let mut found: [Option<Rc<FoundProject>>; N] = [const { None }; N];
            for child in children {
                for (idx, a) in child.iter().enumerate() {
                    if let Some(a) = a {
                        found[idx] = Some(a.clone());
                    }
                }
            }
            if let Some(idx) = targets.iter().position(|target| target == &name.as_str()) {
                found[idx] = Some(me)
            }
            found
        },
        |_, path, parent| FoundProject {
            path: path.clone(),
            parent,
        },
        config_dir,
        false,
    )
}

pub fn get_project_tree(config_dir: &PathBuf, collect_folders: bool) -> ProjectTreeNode {
    recursion_fn(
        |prj_name, children, path, _, folders| ProjectTreeNode {
            prj_name,
            children,
            path,
            folders: (!folders.is_empty()).then_some(folders),
        },
        |_, _, _| (),
        config_dir,
        collect_folders,
    )
}

pub fn get_old_config_file_path(config_dir: &Path) -> Option<PathBuf> {
    let path = path_from_iter([config_dir, PathBuf::from("wechsel_projects.json").as_path()]);
    path.exists().then_some(path)
}
