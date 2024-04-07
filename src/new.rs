use std::{fs, io, path::PathBuf};

use crate::{Args, Config, Project};

pub fn new_prj(args: &Args, config: &mut Config, prj_name: &str, folders: Vec<String>, path: String, parent: String) -> io::Result<()> {
    println!("Creating Project {:?}", args.project_name);
        
    //get parent url
    let parent_url = config.all_prjs.walk(&parent, &(),
        |p, _| p.path.clone(), 
        |p, _, child_path, _| format!("{}/{}", p.path, child_path))
        .expect("Could not find parent path");

    let mut new_pr_path = PathBuf::from(&parent_url);
    new_pr_path.push(path);

    //Create Project Folder
    if !new_pr_path.exists() {
        fs::create_dir(&new_pr_path)
           .expect(format!("Could not create project folder: {}", new_pr_path.to_str().unwrap()).as_str());
    }

    //Create Subfolders
    for subfolder in folders.iter() {
        new_pr_path.push(subfolder);
        
        fs::create_dir(&new_pr_path)
            .expect(format!("Could not create Project Subfolder: {}", new_pr_path.to_str().unwrap()).as_str());

        new_pr_path.pop();
    }
    
    //Insert new Project into config
    config.all_prjs.walk(&parent, &(prj_name, folders),
        |p, (name, folders)| {
            p.children.push(Project {
                name: name.to_string(),
                path: name.to_string(),
                folder: folders.clone(),
                children: vec![] });
        }, 
        |_, _, _, _| ());
    Ok(())
}