# hematite [![Build Status](https://travis-ci.org/PistonDevelopers/hematite.svg?branch=master)](https://travis-ci.org/PistonDevelopers/hematite)

A simple Minecraft written in Rust with the Piston game engine

## Getting Started
`cargo build --release`

## How To Open a World
*This method is only for personal use. Never distribute copyrighted content from Minecraft.*

`<version> = 1.8-pre2`

### OSX

Minecraft stores data in the folder `/Users/<username>/Library/Application Support/minecraft`

* In the Minecraft Launcher, click the button "New Profile"
* Type in a profile name, for example "experimental"
* Check "Enable experimental development versions"
* A message warns you about keeping backups of your worlds. Click "Yes" (remember to do backups)
* In the drop down "use version", select `<version>`
* Click "Save Profile"
* Click "Play" (this will download the snapshot)
* Quit Minecraft
* Copy `versions/<version>/<version>.jar` to the assets folder in Hematite
* Rename the file extension to `.zip`
* Extract the jar
* Copy the `minecraft` folder from the new extracted folder and put it in the Hematite assets folder
* Run `./target/hematite "/Users/<username>/Library/Application Support/minecraft/saves/<world>"`
