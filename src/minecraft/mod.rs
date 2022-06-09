pub mod biome;
pub mod block_state;
pub mod data_1_8_pre2;
pub mod model;
pub mod nbt;
pub mod region;

pub use crate::minecraft::data_1_8_pre2 as data;
use std::env;
use std::fs::{self, File};
use std::io;
use std::path::PathBuf;
use zip::ZipArchive;

fn var(key: &str) -> String {
    match env::var(key) {
        Ok(val) => val,
        Err(err) => panic!("couldn't find env var {}; err: {:?}", key, err),
    }
}

#[cfg(windows)]
pub fn vanilla_root_path() -> PathBuf {
    let appdata = var("appdata");
    let mut buf = PathBuf::from(&appdata);
    buf.push(".minecraft");
    buf
}
#[cfg(target_os = "linux")]
#[must_use]
pub fn vanilla_root_path() -> PathBuf {
    let home = var("HOME");
    let mut buf = PathBuf::from(home);
    buf.push(".minecraft");
    buf
}
#[cfg(target_os = "macos")]
pub fn vanilla_root_path() -> PathBuf {
    let home = var("HOME");
    let mut buf = PathBuf::from(home);
    buf.push("Library");
    buf.push("Application Support");
    buf.push("minecraft");
    buf
}

pub fn fetch_assets(version: &str) {
    let mut buf = vanilla_root_path();
    buf.push("versions");
    buf.push(version);
    buf.push(format!("{}.jar", version));

    println!("Opening {:?}...", &buf);
    let file = File::open(&buf).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    println!("File {:?} contains {} files.", &buf, archive.len());

    let mut count = 0;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        // Won't work because file.name() borrows file 'a lifetime.
        // let path = &PathBuf::from(file.name().trim_right_matches('\0'));
        let path = sanitize_filename(file.name());
        if file.name().starts_with("assets/minecraft") {
            fs::create_dir_all(path.parent().unwrap()).unwrap();

            let mut outfile = File::create(&path).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
            count += 1;
        }
    }

    println!("Extracted {} files.", count);
}

fn sanitize_filename(filename: &str) -> PathBuf {
    // PathBuf::from(filename.trim_right_matches('\0'))
    let no_null_filename = match filename.find('\0') {
        Some(index) => &filename[0..index],
        None => filename,
    };
    PathBuf::from(no_null_filename)
}
