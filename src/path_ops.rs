use eyre::{bail, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::args::Args;

pub fn ensure_valid_paths(args: &Args) -> Result<PathBuf> {
    for source in &args.source {
        if !Path::new(&source).exists() {
            bail!("No such source file {}.", source);
        }
    }

    if args.source.len() > 1 && !Path::new(&args.destination).exists() {
        if args.force {
            fs::create_dir_all(&args.destination).unwrap();
        } else {
            bail!("No such directory {}.", args.destination);
        }
    }

    let destination_path = if Path::new(&args.destination).exists() {
        PathBuf::from(&args.destination).canonicalize().unwrap()
    } else {
        if args.destination.ends_with('/') && args.force {
            fs::create_dir_all(&args.destination).unwrap();
        }
        PathBuf::from(&args.destination)
    };

    if let Some(parent) = Path::new(&args.destination).parent() {
        if args.force && !parent.exists() {
            fs::create_dir_all(parent).unwrap();
        }
    }

    Ok(destination_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use tempfile::TempDir;

    #[test]
    fn test_ensure_valid_paths_valid() {
        // Setup a valid directory structure using TempDir
        let temp_dir = TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("test.txt");
        std::fs::write(&temp_file, b"").unwrap();

        // Create a dummy Args struct
        let args = Args {
            source: vec![temp_file.to_str().unwrap().to_string()],
            destination: temp_dir.path().to_str().unwrap().to_string(),
            force: false,
            no_progress: false,
            verify: false,
            symlink: false,
            hard_link: false,
            keep_display_awake: false,
            no_keep_awake: true,
        };

        let result = ensure_valid_paths(&args);
        println!("Result: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ensure_valid_paths_nonexistent_source() {
        let args = Args {
            source: vec!["/non/existent/file.txt".to_string()],
            destination: "/tmp".to_string(),
            force: true,
            no_progress: false,
            verify: false,
            symlink: false,
            hard_link: false,
            keep_display_awake: false,
            no_keep_awake: true,
        };

        let result = ensure_valid_paths(&args);
        println!("Result: {:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn test_ensure_valid_paths_create_destination() {
        // Create a temporary directory for the source file
        let temp_dir = TempDir::new().unwrap();

        // Create a temporary source file
        let temp_file = temp_dir.path().join("source_file.txt");
        std::fs::write(&temp_file, "dummy content").unwrap(); // Create a dummy file

        // Create a temporary directory for the destination
        let random_dir_name: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(10)
            .map(char::from) // 10-character random string
            .collect();
        let destination_dir = temp_dir.path().join(random_dir_name);
        let destination_dir = destination_dir.to_str().unwrap().to_string() + "/";

        // Create the Args struct with the updated paths
        let args = Args {
            source: vec![temp_file.to_str().unwrap().to_string()],
            destination: destination_dir.clone(),
            force: true,
            no_progress: false,
            verify: false,
            symlink: false,
            hard_link: false,
            keep_display_awake: false,
            no_keep_awake: true,
        };

        // Ensure that the destination directory doesn't exist before the test
        let destination_dir = Path::new(&destination_dir);
        assert!(
            !&destination_dir.exists(),
            "Destination directory should not exist before test."
        );

        // Run the function and check the result
        let result = ensure_valid_paths(&args);
        println!("Result: {:?}", result);

        // Ensure the result is Ok and the destination directory was created
        assert!(result.is_ok());
        assert!(
            destination_dir.exists(),
            "Destination directory should be created at {}.",
            destination_dir.display()
        );
    }
}
