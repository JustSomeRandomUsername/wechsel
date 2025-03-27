use std::{fs, path::PathBuf, rc::Rc};

use serde::Serialize;

use crate::{utils::is_entry_folder_with_extension, PROJECT_EXTENSION, WECHSEL_FOLDER_EXTENSION};

#[derive(Serialize)]
pub struct TreeOutput {
    pub tree: ProjectTreeNode,
    pub active: String,
}
#[derive(Debug, Serialize)]
pub struct ProjectTreeNode {
    #[serde(rename(serialize = "name"))]
    pub prj_name: String,
    pub children: Vec<ProjectTreeNode>,
    pub path: PathBuf,
}

pub struct FoundProject {
    pub parent: Option<Rc<FoundProject>>,
    pub path: PathBuf,
}

fn recursion_fn<
    Out,
    Parent,
    F: Fn(String, Vec<Out>, PathBuf, Rc<Parent>) -> Out,
    F2: Fn(&String, &PathBuf, Option<Rc<Parent>>) -> Parent,
>(
    lambda: F,
    lambda_parent: F2,
) -> Out {
    let home = dirs::home_dir().expect("Could not find home directory");

    fn inner<
        Out,
        Parent,
        F: Fn(String, Vec<Out>, PathBuf, Rc<Parent>) -> Out,
        F2: Fn(&String, &PathBuf, Option<Rc<Parent>>) -> Parent,
    >(
        path: PathBuf,
        depth: usize,
        lambda: &F,
        parent: Option<Rc<Parent>>,
        lambda_parent: &F2,
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

        // is set to true after depth 0 to stop all the checks that are not needed after depth 0
        let mut has_wechsel_folder = depth != 0;

        let mut children: Vec<_> = fs::read_dir(&path)
            .map(|children| {
                children
                    .into_iter()
                    .filter_map(|child| {
                        if !has_wechsel_folder
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
                            )
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        if !has_wechsel_folder && children.len() == 1 {
            // Extra rule for depth 0, if there is only one child project and no wechsel folders take it out of the tree
            children.remove(0)
        } else {
            lambda(prj_name, children, path, parent_out)
        }
    }
    inner(home, 0, &lambda, None, &lambda_parent)
}

pub fn search_for_projects<const N: usize>(targets: [&str; N]) -> [Option<Rc<FoundProject>>; N] {
    recursion_fn(
        |name, children: Vec<[Option<Rc<FoundProject>>; N]>, _, me| {
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
    )
}

pub fn get_project_tree() -> ProjectTreeNode {
    recursion_fn(
        |prj_name, children, path, _| ProjectTreeNode {
            prj_name,
            children,
            path,
        },
        |_, _, _| (),
    )
}
// pub fn get_project_tree(targets: Vec<&str>) -> ProjectTreeNode {
//     let home = dirs::home_dir().expect("Could not find home directory");
//     let mut links = vec![];

//     fn inner_get_project_tree<'a>(
//         path: PathBuf,
//         mut targets: Option<Vec<&'a str>>,
//         links: &mut Vec<Vec<Rc<ProjectTreeNode>>>,
//         depth: usize,
//     ) -> (Rc<ProjectTreeNode>, Option<Vec<&'a str>>) {
//         let prj_name = if depth == 0 {
//             "home".to_string()
//         } else {
//             path.file_stem()
//                 .and_then(|name| name.to_str())
//                 .unwrap_or_default()
//                 .to_string()
//         };

//         let mut found_target = false;
//         if let Some(ref mut targets) = targets {
//             if targets.contains(&prj_name.as_str()) {
//                 targets.retain(|target| target != &prj_name);
//                 found_target = true;
//             }
//             if targets.is_empty() {
//                 return (
//                     Rc::new(ProjectTreeNode {
//                         prj_name,
//                         path,
//                         children: vec![],
//                     }),
//                     Some(vec![]),
//                 );
//             }
//         }

//         // is set to true after depth 0 to stop all the checks that are not needed after depth 0
//         let mut has_wechsel_folder = depth != 0;
//         let mut targets = Some(targets);
//         let mut children: Vec<Rc<ProjectTreeNode>> = fs::read_dir(&path)
//             .map(|children| {
//                 children
//                     .into_iter()
//                     .filter_map(|child| {
//                         if !has_wechsel_folder
//                             && is_entry_folder_with_extension(&child, WECHSEL_FOLDER_EXTENSION)
//                                 .is_some()
//                         {
//                             has_wechsel_folder = true;
//                         }
//                         is_entry_folder_with_extension(&child, PROJECT_EXTENSION).map(|child| {
//                             let (node, new_targets) = inner_get_project_tree(
//                                 child.path().clone(),
//                                 targets.take().unwrap(),
//                                 links,
//                                 depth + 1,
//                             );
//                             targets.replace(new_targets);
//                             node
//                         })
//                     })
//                     .collect()
//             })
//             .unwrap_or_default();

//         let new_node = if !has_wechsel_folder && children.len() == 1 {
//             // Extra rule for depth 0, if there is only one child project and no wechsel folders take it out of the tree
//             children.remove(0)
//         } else {
//             Rc::new(ProjectTreeNode {
//                 prj_name,
//                 path,
//                 children,
//             })
//         };

//         if found_target {
//             links.push(vec![new_node.clone()]);
//         }

//         (new_node, targets.unwrap())
//     }
//     Rc::try_unwrap(
//         inner_get_project_tree(
//             home,
//             if targets.is_empty() {
//                 None
//             } else {
//                 Some(targets)
//             },
//             &mut links,
//             0,
//         )
//         .0,
//     )
//     .unwrap()
// }
