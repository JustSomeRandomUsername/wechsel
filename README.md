# Wechsel
Organise your projects by replacing user folders with symlinks to project folders.

Wechsel is a simple tool that helps you by creating individual Download, Desktop, ... folders for each project.
It replaces the original folders with symlinks to the folders of the current active project.
Like this the random files you download, will be placed in the Download folder they belong to.

Additionaly each project can have init scripts that allow you to do things like automaticly sourcing python enviroments in your python projects.

## How it works
Wechsel stores all the information about your project in a config fille called ```wechsel_projects.json```.
This file contains a tree structure of all your project.
Every project is an entry in that tree that holds information about the project like the name of the project, the path of the project folder of the list of its children.

When switching project this file is read to find out which folders are going to be symlinked and what the symlink targets should be.

If a project dosnt't have a folder that a parent project does have, the folder of the parent project is used. E.g. you project ```uni``` does not have a ```Music``` folder but your root project does, then when swiching to the ```uni``` project the ```Music``` folder of the root project will be symlinked.

## Gnome
There is an acompanying [gnome extension](https://github.com/JustSomeRandomUsername/wechsel-extension) that integrates Wechsel into the gnome shell.

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