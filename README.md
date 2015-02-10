# hematite [![Build Status](https://travis-ci.org/PistonDevelopers/hematite.svg?branch=master)](https://travis-ci.org/PistonDevelopers/hematite)

A simple Minecraft written in Rust with the Piston game engine

![screenshot](./screenshot.png)

## Getting Started

### Windows

* Download SDL2 binaries from <https://www.libsdl.org/download-2.0.php>
* Copy SDL2.dll to `C:\Rust\bin\rustlib\x86_64-pc-windows-gnu\lib`, also in Hematite's root folder.

### OS X

`$ brew install sdl2`

### Ubuntu

`$ sudo apt-get install libsdl2-dev`

Should get you going without problems, if you find any issues please file them.

## How To Open a World

*This method is only for personal use. Never distribute copyrighted content from Minecraft.*

`<version> = 1.8.1`

* In the Minecraft Launcher, click the button "New Profile"
* Type in a profile name, for example "experimental"
* Check "Enable experimental development versions"
* A message warns you about keeping backups of your worlds. Click "Yes" (remember to do backups)
* In the drop down "use version", select `<version>`
* Click "Save Profile"
* Click "Play" (this will download the snapshot)
* Quit Minecraft
* Check out where is your Minecraft folder located (section below)
* Copy `<minecraft_folder>/versions/<version>/<version>.jar` to the assets folder in Hematite
* Rename the file to `<version>.zip` and open it
* Copy the `minecraft` folder from the new zip file put it in the Hematite `assets` folder
* Optional: you can remove `<version>.zip` it's not required anymore
* Run hematite with: `cargo run --release "<path_to_minecraft_world>"`

### Windows

Minecraft folder: `%appdata\minecraft`

Worlds folder: `%appdata\minecraft\saves\<world>`

### OS X

Minecraft folder: `/Users/<username>/Library/Application Support/minecraft`

Worlds folder: `/Users/<username>/Library/Application Support/minecraft/saves/<world>`

### Linux

Minecraft folder: `~/.minecraft`

Worlds folder: `~/.minecraft/saves/<world>`

## Dependencies

![dependencies](./Cargo.png)

[How to contribute](https://github.com/PistonDevelopers/piston/blob/master/CONTRIBUTING.md)
