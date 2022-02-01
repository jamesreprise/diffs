use env_logger::Env;
use log::{error, info, warn};
use rayon::prelude::*;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::fs::ReadDir;
use std::hash::Hasher;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::process;

pub fn run() {
    // Output info-level logging to stdin by default.
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args: Vec<String> = env::args().collect();

    // Should switch to some better env arg parsing.
    if args.len() < 3 {
        println!("{}", "=".repeat(30));
        println!("diffs: diff directories");
        println!("Syntax: ./ddiff <old directory> <new directory>");
        println!("{}", "=".repeat(30));
        error!("Not enough arguments. Exiting...");
        process::exit(1);
    }

    let old_dir = Path::new(&args[1]);
    let new_dir = Path::new(&args[2]);

    if !old_dir.is_dir() {
        error!("{:?} is not a directory.", old_dir);
        process::exit(1);
    }
    if !new_dir.is_dir() {
        error!("{:?} is not a directory.", new_dir);
        process::exit(1);
    }

    diff(old_dir, new_dir);
}

fn diff(old_dir: &Path, new_dir: &Path) {
    let old_dir = old_dir.read_dir().unwrap();
    let old_files = files_from_dir(old_dir);

    if old_files.len() == 0 {
        warn!("The first directory has no files contained within it.");
    } else {
        info!("There are {} files in the first directory.", {
            &old_files.len()
        });
    }

    let old_hashmap = hash_files(old_files);

    let new_dir = new_dir.read_dir().unwrap();
    let new_files: Vec<PathBuf> = files_from_dir(new_dir);

    if new_files.len() == 0 {
        warn!("The second directory has no files contained within it.");
    } else {
        info!("There are {} files in the second directory.", {
            &new_files.len()
        });
    }

    let new_hashmap = hash_files(new_files);

    if new_hashmap == old_hashmap {
        info!("The files contained within these directories are identical, though the directory structure may be different.");
    } else {
        info! {"The following files are present in second directory and not in the first."}
        for k in new_hashmap.keys() {
            if !old_hashmap.contains_key(k) {
                let name = &new_hashmap[k];
                info!("{name}, Sea: {k}");
            }
        }

        info! {"The following files are present in first directory and not in the second."}
        for k in old_hashmap.keys() {
            if !new_hashmap.contains_key(k) {
                let name = &old_hashmap[k];
                info!("{name}, Hash: {k}");
            }
        }
    }
}

fn files_from_dir(dir: ReadDir) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();
    for file in dir {
        if let Ok(file) = file {
            if file.file_type().unwrap().is_file() {
                files.push(file.path());
            } else if file.file_type().unwrap().is_dir() {
                // Extract all files, even from directories.
                // Doesn't follow symlinks.
                let rdir = file.path().read_dir().unwrap();
                files.append(&mut files_from_dir(rdir));
            }
        }
    }
    files
}

fn hash_files(files: Vec<PathBuf>) -> HashMap<String, String> {
    files
        .par_iter()
        .map(|p| {
            (
                hash_file(p.to_path_buf()),
                p.clone().into_os_string().into_string().unwrap(),
            )
        })
        .collect()
}

fn hash_file(path: PathBuf) -> String {
    let mut hasher: seahash::SeaHasher = seahash::SeaHasher::new();
    let file = File::open(path).unwrap();
    let mut br = BufReader::new(file);
    loop {
        let bytes = br.fill_buf().unwrap();
        let len = bytes.len();
        if len == 0 {
            break;
        } else {
            hasher.write(bytes);
            br.consume(len);
        }
    }
    let hash = hasher.finish();
    format!("{:X}", hash)
}
