# Wechsel
Organise your projects by replacing user folders with symlinks to project folders.

Anoyed by the mess in your Download or Desktop folder?

Wechsel is a simple tool that helps you by giving you an individual Download, Desktop, ... folder for each project.
It replaces the original folders with symlinks to the folders of the current active project.
Like this the random files you download, they will be placed in the Download folder they belong to.

Additionaly each project can have init scripts that allow you to do things like automaticly sourcing python enviroments in your python projects.

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