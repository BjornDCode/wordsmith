use std::{env, path::Path};
use fs_extra::dir::{self, CopyOptions};

fn main() {
    // Tell Cargo to rerun this script if resources directory changes
    println!("cargo:rerun-if-changed=resources/");

    // Get the output directory from Cargo
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir).ancestors().nth(3).unwrap();
    let resources_target_dir = target_dir.join("resources");

    // Create copy options
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    copy_options.copy_inside = true;

    // Copy the resources directory to the build output
    let resources_source_dir = Path::new("resources");
    
    // Make sure the target directory exists
    if !resources_target_dir.exists() {
        std::fs::create_dir_all(&resources_target_dir).unwrap();
    }
    
    // Copy the contents (not the directory itself)
    let resources = std::fs::read_dir(resources_source_dir).unwrap();
    for entry in resources {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap();
                let target = resources_target_dir.join(dir_name);
                dir::copy(&path, resources_target_dir.clone(), &copy_options).unwrap_or_else(|_| {
                    println!("Warning: Failed to copy directory {:?}", path);
                    0
                });
            } else {
                let file_name = path.file_name().unwrap();
                let target = resources_target_dir.join(file_name);
                std::fs::copy(&path, &target).unwrap_or_else(|_| {
                    println!("Warning: Failed to copy file {:?}", path);
                    0
                });
            }
        }
    }

    println!("Resources copied to: {:?}", resources_target_dir);
}