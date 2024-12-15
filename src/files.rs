use std::path::Path;

use eyre::{bail, Result};
use fs_extra::dir::CopyOptions;
use fs_extra::dir::{self, TransitState};
use indicatif::ProgressBar;

use crate::cli::Args;

pub fn perform_copy_operation(
    args: &Args,
    source_path: &Path,
    destination_path: &Path,
    pb: &Option<ProgressBar>,
) -> Result<()> {
    // Perform the copy operation
    if source_path.is_dir() {
        copy_dir(args, source_path, destination_path, pb)?;
    } else {
        copy_file(args, source_path, destination_path, pb)?;
    }
    if let Some(pb) = pb {
        pb.finish();
    }
    Ok(())
}

pub fn copy_dir<P: AsRef<Path>>(
    args: &Args,
    source_path: P,
    destination_path: P,
    pb: &Option<ProgressBar>,
) -> Result<()> {
    #[allow(unused)]
    let dir_progress_handler = |info: fs_extra::dir::TransitProcess| {
        let progress = info.copied_bytes * 100 / info.total_bytes;
        if let Some(pb) = &pb {
            pb.set_length(info.total_bytes);
            pb.set_position(info.copied_bytes);
        };

        // Exists
        if info.state == TransitState::Exists && args.force {
            return fs_extra::dir::TransitProcessResult::Overwrite;
        }
        // Access denied
        else if info.state == TransitState::NoAccess && args.force {
            tracing::warn!("Access denied to file {}. Skipping...", info.file_name);
            return fs_extra::dir::TransitProcessResult::Skip;
        }
        // Normal
        if args.force {
            fs_extra::dir::TransitProcessResult::Overwrite
        } else {
            fs_extra::dir::TransitProcessResult::ContinueOrAbort
        }
    };

    // Set up copy options
    let mut options = CopyOptions::new();
    options.overwrite = args.force;
    options.copy_inside = true;
    if !args.no_progress {
        dir::copy(source_path, destination_path, &options)?;
    } else {
        dir::copy_with_progress(
            source_path,
            destination_path,
            &options,
            dir_progress_handler,
        )?;
    };
    Ok(())
}

pub fn copy_file<P: AsRef<Path>>(
    args: &Args,
    source_path: P,
    destination_path: P,
    pb: &Option<ProgressBar>,
) -> Result<()> {
    #[allow(unused)]
    let file_progress_handler = |info: fs_extra::file::TransitProcess| {
        let progress = info.copied_bytes * 100 / info.total_bytes;
        if let Some(pb) = &pb {
            pb.set_length(info.total_bytes);
            pb.set_position(info.copied_bytes);
        };
    };

    if destination_path.as_ref().exists() && !args.force {
        bail!(
            "Fail already exists at {}",
            destination_path.as_ref().display()
        )
    }
    if args.hard_link {
        std::fs::hard_link(source_path, destination_path)?;
    } else if args.symlink {
        #[cfg(unix)]
        std::os::unix::fs::symlink(source_path, destination_path)?;
        #[cfg(windows)]
        std::os::windows::fs::symlink(source_path, destination_path)?;
    } else {
        let mut file_options = fs_extra::file::CopyOptions::new();
        file_options.overwrite = args.force;
        if args.reflink {
            reflink_copy::reflink_or_copy(source_path, destination_path)?;
        } else if !args.no_progress {
            fs_extra::file::copy_with_progress(
                source_path,
                destination_path,
                &file_options,
                file_progress_handler,
            )?;
        } else {
            fs_extra::file::copy(source_path, destination_path, &file_options)?;
        }
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    // Helper function to create a file with some content
    fn create_test_file<P: AsRef<Path>>(path: P) -> Result<()> {
        let mut file = File::create(path)?;
        writeln!(file, "Hello, world!")?;
        Ok(())
    }

    #[test]
    fn test_copy_file_success() {
        let temp_dir = tempdir().unwrap();
        let source_file = temp_dir.path().join("source.txt");
        let dest_file = temp_dir.path().join("destination.txt");

        // Create a source file
        create_test_file(&source_file).unwrap();

        // Prepare arguments
        let args = Args {
            force: true,
            no_progress: true,
            symlink: false,
            hard_link: false,
            destination: dest_file.to_str().unwrap().to_string(),
            keep_display_awake: false,
            no_keep_awake: true,
            source: vec![source_file.to_str().unwrap().to_string()],
            verify: false,
            reflink: false,
        };

        // Perform the copy operation
        perform_copy_operation(
            &args,
            &source_file,
            &dest_file,
            &None, // No progress bar
        )
        .unwrap();

        // Verify the destination file exists and has the same content
        assert!(dest_file.exists());
        let content = fs::read_to_string(dest_file).unwrap();
        assert_eq!(content, "Hello, world!\n");
    }

    #[test]
    fn test_copy_directory_success() {
        let temp_dir = tempdir().unwrap();
        let source_dir = temp_dir.path().join("source");
        let dest_dir = temp_dir.path().join("destination");

        // Create a source directory with a file
        fs::create_dir_all(&source_dir).unwrap();
        let source_file = source_dir.join("file.txt");
        create_test_file(&source_file).unwrap();

        // Prepare arguments
        let args = Args {
            force: true,
            no_progress: true,
            symlink: false,
            hard_link: false,
            destination: dest_dir.to_str().unwrap().to_string(),
            keep_display_awake: false,
            no_keep_awake: true,
            source: vec![source_dir.to_str().unwrap().to_string()],
            verify: false,
            reflink: false,
        };

        // Perform the copy operation
        perform_copy_operation(
            &args,
            &source_dir,
            &dest_dir,
            &None, // No progress bar
        )
        .unwrap();

        // Verify the destination directory and file exist
        assert!(dest_dir.exists());
        assert!(dest_dir.join("file.txt").exists());
    }
}
