# hematite [![Build Status](https://travis-ci.org/PistonDevelopers/hematite.svg?branch=master)](https://travis-ci.org/PistonDevelopers/hematite)

A simple Minecraft written in Rust with the Piston game engine

![screenshot](./screenshot.png)

## How To Open a World

*This method is only for personal use. Never distribute copyrighted content from Minecraft.*

* In the Minecraft Launcher, click the button “New Profile”
* In the drop down "use version", select `1.8.8`
* Click “Save Profile”
* Click “Play” (this will download the snapshot)
* Quit Minecraft

* **Copy** your world save to to the hematite directory (It may corrupt your world)
* Save Locations:
  * **Windows:** `%appdata%\.minecraft\saves\`
  * **OSX:** `~/Library/Application Support/minecraft/saves/`
  * **Linux/Other:** `~/.minecraft/saves/`
* Run hematite with: `cargo run --release "./<WORLD_NAME>"`

## Dependencies

![dependencies](./Cargo.png)

[How to contribute](https://github.com/PistonDevelopers/piston/blob/master/CONTRIBUTING.md)
