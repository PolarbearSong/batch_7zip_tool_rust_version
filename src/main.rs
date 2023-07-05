use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;

fn main() {
    let program_files = match env::var("ProgramFiles") {
        Ok(val) => val,
        Err(_) => {
            println!("ProgramFiles environment variable not found.");
            return;
        }
    };

    let seven_zip_path = PathBuf::from(&program_files).join("7-Zip").join("7z.exe");

    if !seven_zip_path.exists() {
        println!("7-Zip utility not found. Please ensure it is installed correctly.");
        return;
    }

    let compression_level = "7";

    let args: Vec<String> = env::args().skip(1).collect();

    let num_threads = 2;

    let handles: Vec<_> = args
        .into_iter()
        .take(num_threads)
        .map(|arg| {
            let seven_zip_path = seven_zip_path.clone();
            thread::spawn(move || {
                if let Ok(metadata) = fs::metadata(&arg) {
                    if metadata.is_file() {
                        compress_file(&seven_zip_path, &compression_level, &arg);
                    } else if metadata.is_dir() {
                        compress_directory(&seven_zip_path, &compression_level, &arg);
                    } else {
                        println!("Invalid argument: {}", arg);
                    }
                } else {
                    println!("Invalid argument: {}", arg);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

fn compress_file(seven_zip_path: &PathBuf, compression_level: &str, file_path: &str) {
    let output = Command::new(seven_zip_path)
        .args(&["a", "-tzip", &format!("-mx={}", compression_level), &format!("{}.zip", file_path), file_path])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                println!("Compression succeeded for file: {}", file_path);
            } else {
                println!("Compression failed for file: {}", file_path);
            }
        }
        Err(err) => {
            println!("Failed to execute 7-Zip: {}", err);
        }
    }
}

fn compress_directory(seven_zip_path: &PathBuf, compression_level: &str, dir_path: &str) {
    let output = Command::new(seven_zip_path)
        .args(&["a", "-tzip", &format!("-mx={}", compression_level), &format!("{}.zip", dir_path), dir_path])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                println!("Compression succeeded for directory: {}", dir_path);
            } else {
                println!("Compression failed for directory: {}", dir_path);
            }
        }
        Err(err) => {
            println!("Failed to execute 7-Zip: {}", err);
        }
    }
}
