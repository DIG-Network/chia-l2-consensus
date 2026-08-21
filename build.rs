//! Build script: recompile Rue puzzles when source changes.
//!
//! Runs `rue build --hex` and `rue build --hash` for each .rue file in puzzles/,
//! writing output to puzzles/compiled/. Cargo reruns this when any .rue file changes.

use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let puzzles_dir = Path::new("puzzles");
    let compiled_dir = puzzles_dir.join("compiled");

    // Ensure output directory exists
    fs::create_dir_all(&compiled_dir).expect("Failed to create puzzles/compiled/");

    // Find all .rue files
    let rue_files: Vec<_> = fs::read_dir(puzzles_dir)
        .expect("Failed to read puzzles/")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rue"))
        .collect();

    for entry in &rue_files {
        let rue_path = entry.path();
        let stem = rue_path.file_stem().unwrap().to_str().unwrap();

        // Tell Cargo to rerun if this .rue file changes
        println!("cargo:rerun-if-changed={}", rue_path.display());

        // Compile to hex
        let hex_output = Command::new("rue")
            .args(["build", &rue_path.to_string_lossy(), "--hex"])
            .output();

        match hex_output {
            Ok(output) if output.status.success() => {
                let hex_path = compiled_dir.join(format!("{}.hex", stem));
                fs::write(&hex_path, &output.stdout)
                    .unwrap_or_else(|e| panic!("Failed to write {}: {}", hex_path.display(), e));
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!(
                    "rue build --hex failed for {}: {}",
                    rue_path.display(),
                    stderr
                );
            }
            Err(e) => {
                // rue not installed — skip compilation but warn
                println!(
                    "cargo:warning=rue not found ({}), skipping puzzle compilation for {}",
                    e,
                    rue_path.display()
                );
                continue;
            }
        }

        // Compile to hash
        let hash_output = Command::new("rue")
            .args(["build", &rue_path.to_string_lossy(), "--hash"])
            .output();

        match hash_output {
            Ok(output) if output.status.success() => {
                let hash_path = compiled_dir.join(format!("{}.hash", stem));
                fs::write(&hash_path, &output.stdout)
                    .unwrap_or_else(|e| panic!("Failed to write {}: {}", hash_path.display(), e));
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!(
                    "cargo:warning=rue build --hash failed for {}: {}",
                    rue_path.display(),
                    stderr
                );
            }
            Err(_) => {} // already warned above
        }
    }
}
