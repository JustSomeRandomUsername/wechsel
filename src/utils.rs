use std::{
    fs::{self, DirEntry},
    io::Error,
    path::{Path, PathBuf},
};

use crate::WECHSEL_FOLDER_EXTENSION;

pub const HOME_FOLDERS: [&str; 6] = [
    "Desktop",
    "Downloads",
    "Documents",
    "Pictures",
    "Videos",
    "Music",
];
pub fn get_home_folder_paths<'a>() -> impl Iterator<Item = (&'a str, PathBuf)> {
    [
        (HOME_FOLDERS[0], dirs::desktop_dir()),
        (HOME_FOLDERS[1], dirs::download_dir()),
        (HOME_FOLDERS[2], dirs::document_dir()),
        (HOME_FOLDERS[3], dirs::picture_dir()),
        (HOME_FOLDERS[4], dirs::video_dir()),
        (HOME_FOLDERS[5], dirs::audio_dir()),
    ]
    .into_iter()
    .filter_map(|(name, path)| path.map(|a| (name, a)))
}

pub fn get_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|conf| PathBuf::from_iter([conf, PathBuf::from("wechsel")]))
}

//** Find subfolders of target path that have the wechsel extension*/
pub fn get_folders(path: &PathBuf) -> Vec<PathBuf> {
    fs::read_dir(path)
        .ok()
        .map(|children| {
            children
                .into_iter()
                .filter_map(|file| {
                    is_entry_folder_with_extension(&file, WECHSEL_FOLDER_EXTENSION)
                        .map(|file| file.path())
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn path_from_iter<const N: usize, S: AsRef<Path>>(inp: [S; N]) -> PathBuf {
    inp.into_iter().collect()
}

pub fn is_entry_folder_with_extension<'a>(
    entry: &'a Result<DirEntry, Error>,
    extension: &str,
) -> Option<&'a DirEntry> {
    entry.as_ref().ok().and_then(|entry| {
        (entry.path().is_dir()
            && entry
                .path()
                .extension()
                .map(|ext| ext == extension)
                .unwrap_or(false))
        .then_some(entry)
    })
}

pub fn query_active_project() -> Option<String> {
    let project_folder_path = path_from_iter([
        dirs::home_dir().expect("Could not find home directory"),
        PathBuf::from("Project"),
    ]);

    if project_folder_path.is_symlink() {
        if let Ok(target) = fs::read_link(project_folder_path) {
            return target
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(|a| a.to_string());
        }
    }
    None
}
