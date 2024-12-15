use blake2::{Blake2s256, Digest};
use clap::Parser;
use eyre::{bail, Result};
use fs_extra::dir::{self, CopyOptions, TransitState};
use humansize::{format_size, DECIMAL};
use std::path::Path;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Command-line utility for copying files or directories with optional recursion and overwriting.
#[derive(Parser, Debug)]
#[command(about = "Copies files or directories with options for recursion and overwriting.")]
struct Args {
    /// Source file or directory to copy
    source: String,

    /// Destination file or directory
    destination: String,

    /// Overwrite destination if it exists
    #[arg(short, long)]
    force: bool,

    /// Hard link file
    #[arg(long)]
    hard_link: bool,

    /// Symbol link file
    #[arg(long)]
    #[cfg(unix)]
    symlink: bool,

    /// Verify hash of folder / file once copied
    #[arg(long)]
    verify: bool,

    /// Disable progress bar
    #[arg(long)]
    no_progress: bool,

}

fn main() -> Result<()> {
    // let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| config::DEFAULT_LOG_DIRECTIVE.to_owned());
    let rust_log = std::env::var("RUST_LOG").unwrap_or("INFO".into());
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::new(rust_log))
        .init();

    let args = Args::parse();

    let source_path = Path::new(&args.source);
    let destination_path = Path::new(&args.destination);

    // Check if source exists
    if !source_path.exists() {
        tracing::error!("Error: Source '{}' does not exist.", args.source);
        return Ok(());
    }

    // Check if destination exists
    if destination_path.exists() && !args.force {
        tracing::error!(
            "Error: Destination '{}' already exists. Use -f to overwrite.",
            args.destination
        );
        return Ok(());
    }

    // Set up progress handler
    #[allow(unused)]
    let dir_progress_handler = |info: fs_extra::dir::TransitProcess| {
        tracing::info!(
            "Progress: {}% ({}/{})",
            info.file_bytes_copied * 100 / info.total_bytes,
            format_size(info.file_bytes_copied as u64, DECIMAL),
            format_size(info.total_bytes as u64, DECIMAL)
        );
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

    #[allow(unused)]
    let file_progress_handler = |info: fs_extra::file::TransitProcess| {
        tracing::info!(
            "Progress: {}% ({}/{})",
            info.copied_bytes * 100 / info.total_bytes,
            format_size(info.copied_bytes as u64, DECIMAL),
            format_size(info.total_bytes as u64, DECIMAL)
        );
    };

    // Perform the copy operation
    if source_path.is_dir() {
        // Set up copy options
        let mut options = CopyOptions::new();
        options.overwrite = args.force;
        options.copy_inside = true;
        if args.no_progress {
            dir::copy(
                source_path,
                destination_path,
                &options,
            )?;     
        } else {
            dir::copy_with_progress(
                source_path,
                destination_path,
                &options,
                dir_progress_handler,
            )?;    
        }
        
    } else {
        if destination_path.exists() && !args.force {
            bail!("Fail already exists at {}", destination_path.display())
        }
        if args.hard_link {
            std::fs::hard_link(source_path, destination_path)?;
        }

        #[cfg(unix)]
        if args.symlink {
            std::os::unix::fs::symlink(source_path, destination_path)?;
        } else {
            let mut file_options = fs_extra::file::CopyOptions::new();
            file_options.overwrite = args.force;
            if args.no_progress {
                fs_extra::file::copy(
                    source_path,
                    destination_path,
                    &file_options,
                )?;
            } else {
                fs_extra::file::copy_with_progress(
                    source_path,
                    destination_path,
                    &file_options,
                    file_progress_handler,
                )?;
            }
            
        }
    }

    tracing::info!("Copy completed successfully.");

    if args.verify {
        tracing::info!("Computing hash...");
        let mut source_hash = Blake2s256::new();
        let mut dst_hash = Blake2s256::new();
        if source_path.is_dir() {
            file_hashing::get_hash_folder(source_path, &mut source_hash, num_cpus::get(), |_| {})?;
            file_hashing::get_hash_folder(
                destination_path,
                &mut dst_hash,
                num_cpus::get(),
                |_| {},
            )?;
        } else {
            file_hashing::get_hash_file(source_path, &mut source_hash)?;
            file_hashing::get_hash_file(destination_path, &mut dst_hash)?;
        }
        let source_hash = source_hash.finalize();
        let dst_hash = dst_hash.finalize();
        let source_hash = hex::encode(source_hash);
        let dst_hash = hex::encode(dst_hash);
        tracing::info!("Source hash: {}", source_hash);
        tracing::info!("Destination hash: {}", dst_hash);
        assert_eq!(source_hash, dst_hash);
    }
    Ok(())
}
