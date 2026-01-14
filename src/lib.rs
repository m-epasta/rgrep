use std::env;
use std::error::Error;
use std::path::Path;
use std::process::Command;

/// FROM ROOT: run_rgrep with pipeline `left_command | right_command`
pub fn run_rgrep_from_root(left_command: &str, right_command: &str) -> bool {
    find_rgrep_dir().expect("Failed to change to rgrep directory");

    let pipeline = format!("{} | {}", left_command, right_command);
    let output = Command::new("sh")
        .arg("-c")
        .arg(&pipeline)
        .output()
        .expect("Failed to execute pipeline");

    // Pretty print stdout if any
    if !output.stdout.is_empty() {
        println!("rgrep output:");
        print!("{}\n", String::from_utf8_lossy(&output.stdout));
    }

    // Print stderr if any (for debugging)
    if !output.stderr.is_empty() {
        eprintln!("rgrep stderr:");
        eprint!("{}\n", String::from_utf8_lossy(&output.stderr));
    }

    output.status.success()
}

fn find_rgrep_dir() -> Result<(), Box<dyn Error>> {
    if Path::new("target").exists() {
        // Already in the rgrep directory
        Ok(())
    } else {
        env::set_current_dir("rgrep").map_err(|e| Box::new(e) as Box<dyn Error>)
    }
}

pub fn build_rgrep() -> Result<(), Box<dyn Error>> {
    Command::new("sh")
        .arg("-c")
        .arg("cargo build --release")
        .status()
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;

    Ok(())
}

fn verify_build() -> bool {
    Path::new("./target/release/rgrep").exists()
}

pub fn is_rgrep_built() -> bool {
    verify_build()
}
