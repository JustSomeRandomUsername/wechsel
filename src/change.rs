use std::{collections::HashMap, fs, io, path::PathBuf};

use crate::{get_path, Config};

pub fn change_prj(name: &str, config: &mut Config, config_dir: PathBuf) -> io::Result<(Vec<PathBuf>, HashMap<String, String>)> {
    // Find Project Folder Urls
    let map = config.all_prjs.walk(name, &(),
    |p, _| {
        let new_links = p.folder.iter()
            .map(|f| 
                (PathBuf::from(f).file_name().unwrap().to_str().unwrap().to_owned(), PathBuf::from_iter(vec![&p.path, &f])))
            .collect::<HashMap<String, PathBuf>>();
        
        return (new_links, p.path.clone());
    },
    |p,_,child_links: (HashMap<String, PathBuf>, String),gen| {
        let mut links = gen(p,&()).0;
        links.extend(child_links.0.iter()
            .map(|(k,v)| {
                let mut path = PathBuf::from(&p.path);
                path.push(v);
                return (k.clone(), path);
            }).collect::<HashMap<String, PathBuf>>());
        
        let prj_path: PathBuf = [&p.path, &child_links.1].iter().collect();
        return (links, prj_path.to_str().expect("Could not convert to string").to_owned());
    });

    let scripts = if let Some((map, prj_path)) = map {
        // Link Folders
        for (name, path) in map {
            let mut parking = dirs::home_dir().ok_or(io::Error::new(std::io::ErrorKind::Other, "No Home dir found"))?;
            let mut target = parking.clone();
            target.push(&name);
            parking.push(name+"1"); 
                
            if !path.exists() || (target.exists() && !target.is_symlink()) {
                println!("Could not symlink folder {:?}, target {:?} {:?}", path, parking, target);
                continue;   
            }

            std::os::unix::fs::symlink(&path, &parking).unwrap_or_default();

            fs::rename(&parking, &target)?
            
        }

        let env_vars = HashMap::from_iter(vec![
            ("PRJ".to_owned(), name.to_owned()), 
            ("PRJ_PATH".to_owned(), prj_path.clone()),
            ("OLD_PRJ".to_owned(), config.active.clone()),
            ("OLD_PRJ_PATH".to_owned(), get_path(&mut config.all_prjs, &config.active)
                .and_then(|path| path.to_str().map(|a|a.to_owned())).unwrap_or_default())

        ]);
        // Write Enviroment Variables for Fish
        let mut enviroment_vars = PathBuf::from(&config_dir);
        enviroment_vars.push("enviroment_variables.fish");
        fs::write(enviroment_vars, format!("set -x PRJ {name}\nset -x PRJ_PATH {prj_path}"))?;

        // Write Enviroment Variables for Bash
        let mut enviroment_vars = PathBuf::from(&config_dir);
        enviroment_vars.push("enviroment_variables.sh");
        fs::write(enviroment_vars, format!("export PRJ={name}\nexport PRJ_PATH={prj_path}"))?;
        
        let mut scripts = vec![];
        // Global on change script .config/on-prj-change
        let mut script = PathBuf::from(&config_dir);
        script.push("on-prj-change");
        scripts.push(script);
        
        (scripts, env_vars)
    } else {
        eprintln!("Could not find Project {}", name);
        std::process::exit(1);
    };
        
    config.active = name.to_owned();
    return Ok(scripts);
}