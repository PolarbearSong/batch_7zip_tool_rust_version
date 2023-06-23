use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    // Retrieve the value of the ProgramFiles environment variable
    let program_files = match env::var("ProgramFiles") {
        Ok(val) => val,
        Err(_) => {
            println!("ProgramFiles environment variable not found.");
            return;
        }
    };

    // Construct the path to the 7-Zip executable
    let seven_zip_path = Path::new(&program_files).join("7-Zip").join("7z.exe");

    // Check if 7-Zip executable exists
    if !seven_zip_path.exists() {
        println!("7-Zip utility not found. Please ensure it is installed correctly.");
        return;
    }

    // Set the compression level (0 to 9, where 0 is no compression and 9 is the highest compression)
    let compression_level = "9";

    // Iterate over each argument
    for arg in env::args().skip(1) {
        // Check if the argument exists
        if Path::new(&arg).exists() {
            // Argument exists as either a file or directory
            let output = Command::new(&seven_zip_path)
                .args(&["a", "-tzip", &format!("-mx={}", compression_level), &format!("{}.zip", arg), &arg])
                .output()
                .expect("Failed to execute 7-Zip.");

            if !output.status.success() {
                println!("Compression failed for argument: {}", arg);
            }
        } else {
            // Argument does not exist
            println!("Invalid argument: {}", arg);
        }
    }
}
