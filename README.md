<div>
  <!-- Crates version -->
  <a href="https://crates.io/crates/wechsel">
    <img src="https://img.shields.io/crates/v/wechsel.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
</div>

# Wechsel
Organize your computer by replacing user folders with symlinks to project folders.

Wechsel is a simple tool that helps you by creating individual Download, Desktop, ... folders for each project.
It replaces the original folders with symlinks to the folders of the current active project.
Now the random files you download, will be placed in the Download folder they belong to.

Additionally each project can have init scripts that allow you to do things like automatically sourcing python environments in your python projects.

## How it works
Projects are defined by project folders that have the .p filetype e.g. `~/home.p`.

All Projects are structured in a tree structure with your home directory as the root. Each projects children need to be placed into their parents project folder. e.g. `~/home.p/uni.p`

Each project can have wechsel folders that have the .w filetype. These are the folders that will be symlinked to your home directory when the project gets switched to. `~/home.p/uni.p/Desktop.w`

If a project doesn't have a folder that a parent project does have, the folder of the parent project is used. E.g. you project ```uni``` does not have a ```Music``` folder but the parent project does, then when swiching to the ```uni``` project the ```Music``` folder of the parent project will be symlinked.

## Scripts
Wechsel has a `on-prj-change` and a `on-prj-create` script in `wechsel` folder in your config directory, often `~/.config/wechsel`.

These script get called with some env variables set: `PRJ`, `PRJ_PATH` and for the change script also `OLD_PRJ` and `OLD_PRJ_PATH`.

These script can be used to extend the functionality of wechsel.
Heres a list of some of the things I have been using these for:
- Giving every project its own wallpaper
- Creating a python env for every project and auto sourcing the current one
- Changing git username and email depending on the project
- Connecting to a vpn on project change 

## Gnome
There is an accompanying [gnome extension](https://github.com/JustSomeRandomUsername/wechsel-extension) that integrates Wechsel into the gnome shell.

## Installation
The simplest way to install Wechsel is with ```cargo```:

### Cargo
```cargo install wechsel```

### Github Release
Download the latest release from the [release page](https://github.com/JustSomeRandomUsername/wechsel/releases).
Move the binary to a folder in your PATH and make it executable.


## Setup
Wechsel needs to go through an initial setup to create the necessary folders and files.
To do this run ```wechsel init```. This will guide you through the setup process.

The main thing this will do is creating a folder in your home directory, lets say is called "projects" that will be the place all projects are put. And a root project, lets call it "default" that will be the root of the project tree.

Then it will move the original folders (It will ask you which ones) from your home folder to the root project. And create symlinks in their place.