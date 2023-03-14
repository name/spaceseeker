use std::env;
use std::fs;
use std::fs::metadata;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use csv::{WriterBuilder, Writer};
use chrono::{DateTime, Local, Utc};

fn main() {
    let output_file = fs::File::create("C:\\Temp\\output.csv").unwrap();
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for drive_letter in b'A'..=b'Z' {
        let root_dir = PathBuf::from(format!("{}:\\", drive_letter as char));
        if root_dir.is_dir() {
            write_dir_contents(&root_dir, &mut writer);
        }
    }
}

fn write_dir_contents(dir_path: &Path, writer: &mut Writer<std::fs::File>) -> Option<f64> {
    let paths = match fs::read_dir(dir_path) {
        Ok(paths) => paths,
        Err(err) => {
            //eprintln!("Error reading directory {:?}: {}", dir_path, err);
            return None;
        }
    };

    let mut max_size = 40.0;
    let mut dir_size = 0.0;
    for path in paths {
        let dir_entry = match path {
            Ok(dir_entry) => dir_entry,
            Err(err) => {
                //eprintln!("Error accessing path in directory {:?}: {}", dir_path, err);
                continue;
            }
        };
        let metadata = match metadata(dir_entry.path()) {
            Ok(metadata) => metadata,
            Err(err) => {
                //eprintln!("Error accessing metadata for path {:?}: {}", dir_entry.path(), err);
                continue;
            }
        };
        
        let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
        let modified = metadata.modified().unwrap();
        let modified_str = DateTime::<Local>::from(modified).format("%Y-%m-%d %H:%M").to_string();
        let name = dir_entry.path().display().to_string();

        if metadata.is_file() {
            if size_mb < max_size { continue;  }
            dir_size += size_mb;
            writer.serialize((
                name,
                format!("{:.2} MB", size_mb),
                modified_str,
                "File"
            )).unwrap();
        } else {
            let subdir_size = write_dir_contents(&dir_entry.path(), writer);
            match subdir_size {
                Some(subdir_size) => {
                    dir_size += subdir_size;
                    if subdir_size >= max_size {
                        writer.serialize((
                            name,
                            format!("{:.2} MB", subdir_size),
                            modified_str,
                            "Folder"
                        )).unwrap();
                    }
                }
                None => {
                    if size_mb >= max_size {
                        writer.serialize((
                            name,
                            format!("{:.2} MB", size_mb),
                            modified_str,
                            "Folder"
                        )).unwrap();
                    }
                }
            };
        }
    }
    Some(dir_size)
}