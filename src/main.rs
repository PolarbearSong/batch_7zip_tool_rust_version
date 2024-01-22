use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let program_files = env::var("ProgramFiles").unwrap_or_else(|_| {
        println!("ProgramFiles environment variable not found.");
        std::process::exit(1);
    });

    let seven_zip_path = PathBuf::from(&program_files).join("7-Zip").join("7z.exe");

    if !seven_zip_path.exists() {
        println!("7-Zip utility not found. Please ensure it is installed correctly.");
        std::process::exit(1);
    }

    let compression_level = "7";
    let max_threads = 2; // Maximum number of concurrent threads

    let file_list: Vec<_> = env::args().skip(1).collect();

    let pool = Arc::new(Mutex::new(ThreadPool::new(max_threads)));

    for file in file_list {
        if let Ok(metadata) = fs::metadata(&file) {
            if metadata.is_file() {
                let seven_zip_path_clone = seven_zip_path.clone();
                let compression_level = compression_level.to_string();
                pool.lock().unwrap().execute(move || compress_file(&seven_zip_path_clone, &compression_level, &file));
            } else if metadata.is_dir() {
                let seven_zip_path_clone = seven_zip_path.clone();
                let compression_level = compression_level.to_string();
                pool.lock().unwrap().execute(move || compress_directory(&seven_zip_path_clone, &compression_level, &file));
            } else {
                println!("Invalid argument: {}", file);
            }
        } else {
            println!("Invalid argument: {}", file);
        }
    }

    pool.lock().unwrap().join();

}

fn compress_file(seven_zip_path: &PathBuf, compression_level: &str, file_path: &str) {
    let file_name_without_extension = Path::new(file_path)
        .file_stem()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or(file_path);
    
    let output = Command::new(&seven_zip_path)
        .args(&["a", "-tzip", "-bso2", &format!("-mx={}", compression_level), &format!("{}.zip", file_name_without_extension), file_path])
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
    let output = Command::new(&seven_zip_path)
        .args(&["a", "-tzip", "-bso2" ,&format!("-mx={}", compression_level), &format!("{}.zip", dir_path), dir_path])
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

struct ThreadPool {
    workers: Vec<Worker>,
    sender: std::sync::mpsc::Sender<Job>,
}

impl ThreadPool {
    fn new(max_threads: usize) -> Self {
        assert!(max_threads > 0);

        let (sender, receiver) = std::sync::mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let workers: Vec<_> = (0..max_threads)
            .map(|_| {
                let receiver = Arc::clone(&receiver);
                Worker::new(receiver)
            })
            .collect();

        Self { workers, sender }
    }

    fn execute<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(job);
        self.sender.send(job).expect("Failed to send job to the thread pool.");
    }

    fn join(&mut self) {
        for worker in &mut self.workers {
            worker.join();
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(receiver: Arc<Mutex<std::sync::mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let job = receiver.lock().unwrap().recv().unwrap();
                job();
            }
        });

        Self { thread: Some(thread) }
    }

    fn join(&mut self) {
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}
