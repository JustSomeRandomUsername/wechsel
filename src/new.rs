use std::{fs, io, path::PathBuf};
use crate::{Config, Project};

pub fn new_prj(config: &mut Config, prj_name: &str, folders: Vec<String>, path: String, parent: String) -> io::Result<()> {
    println!("Creating Project {:?}", prj_name);
        
    //get parent url
    let parent_url = config.all_prjs.walk(&parent, &(),
        |p, _| PathBuf::from(&p.path), 
        |p, _, child_path, _| [PathBuf::from(&p.path), child_path].iter().collect())
        .expect("Could not find parent path");

    let mut new_pr_path = PathBuf::from(&parent_url);
    new_pr_path.push(path.clone());

    //Create Project Folder
    if !new_pr_path.exists() {
        fs::create_dir_all(&new_pr_path)
           .expect(format!("Could not create project folder: {}", new_pr_path.to_str().unwrap()).as_str());
    } else {
        println!("Project Folder already exists");
    }

    //Create Subfolders
    for subfolder in folders.iter() {
        new_pr_path.push(subfolder);
        
        if !new_pr_path.exists() {
            fs::create_dir(&new_pr_path)
                .expect(format!("Could not create Project Subfolder: {}", new_pr_path.to_str().unwrap()).as_str());
        }

        new_pr_path.pop();
    }
    
    //Insert new Project into config
    config.all_prjs.walk(&parent, &(prj_name, folders, path),
        |p, (name, folders, path)| {
            p.children.push(Project {
                name: name.to_string(),
                path: path.clone(),
                folder: folders.clone(),
                children: vec![] });
        }, 
        |_, _, _, _| ());
    Ok(())
}