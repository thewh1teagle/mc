use blake2::{Blake2s256, Digest};
use clap::Parser;
use eyre::{bail, Result};
use fs_extra::dir::{self, CopyOptions, TransitState};
use tracing_indicatif::IndicatifLayer;
use std::{fmt::Write, path::Path, time::Instant};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use console::style;

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
    symlink: bool,

    /// Verify hash of folder / file once copied
    #[arg(long)]
    verify: bool,

    /// Disable progress bar
    #[arg(long)]
    no_progress: bool,

    /// Disable keep system awake while copy
    #[arg(long)]
    no_keep_awake: bool,

    /// Keep display awake while copy
    #[arg(long)]
    keep_display_awake: bool,
}

fn main() -> Result<()> {
    let start_time = Instant::now();

    // let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| config::DEFAULT_LOG_DIRECTIVE.to_owned());
    let rust_log = std::env::var("RUST_LOG").unwrap_or("INFO".into());
    let indicatif_layer: IndicatifLayer<tracing_subscriber::registry::Registry> = IndicatifLayer::new();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
        .with(EnvFilter::new(rust_log))
        .init();

    let args = Args::parse();
    

    if !args.no_keep_awake {
        // Stopped when program ends
        keepawake::Builder::default()
            .app_name(env!("CARGO_PKG_NAME"))
            .display(args.keep_display_awake)
            .idle(true);
    }

    let source_path = Path::new(&args.source);
    let mut destination_path = if args.destination == "." {
        std::fs::canonicalize(".")?
    } else {
        Path::new(&args.destination).to_path_buf()
    };

    if source_path.is_file() && destination_path.is_dir() {
        destination_path = destination_path.join(source_path.file_name().unwrap());
    }
    let destination_path = destination_path.as_path();
    tracing::debug!("destination: {}", destination_path.display());

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

    let pb = ProgressBar::new(0);
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta_precise})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));

    // Set up progress handler
    #[allow(unused)]
    let dir_progress_handler = |info: fs_extra::dir::TransitProcess| {
        let progress = info.copied_bytes * 100 / info.total_bytes;
        pb.set_length(info.total_bytes);
        pb.set_position(info.copied_bytes);
        // tracing::info!(
        //     "Progress: {}% ({}/{})",
        //     progress,
        //     format_size(info.copied_bytes as u64, DECIMAL),
        //     format_size(info.total_bytes as u64, DECIMAL)
        // );
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
        let progress = info.copied_bytes * 100 / info.total_bytes;
        pb.set_length(info.total_bytes);
        pb.set_position(info.copied_bytes);
        // tracing::info!(
        //     "Progress: {}% ({}/{})",
        //     progress,
        //     format_size(info.copied_bytes as u64, DECIMAL),
        //     format_size(info.total_bytes as u64, DECIMAL)
        // );
    };

    // Perform the copy operation
    println!(
        "{}",
        style(format!("Copy {} to {}...", source_path.display(), destination_path.display())).bold().dim(),
    );
    if source_path.is_dir() {
        // Set up copy options
        let mut options = CopyOptions::new();
        options.overwrite = args.force;
        options.copy_inside = true;
        if args.no_progress {
            dir::copy(source_path, destination_path, &options)?;
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
        } else if args.symlink {
            #[cfg(unix)]
            std::os::unix::fs::symlink(source_path, destination_path)?;
            #[cfg(not(unix))]
            panic!("This platform doesn't support symlink");
        } else {
            let mut file_options = fs_extra::file::CopyOptions::new();
            file_options.overwrite = args.force;
            if args.no_progress {
                fs_extra::file::copy(source_path, destination_path, &file_options)?;
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

    pb.finish();
    println!("{}", style(format!("Copy completed successfully in {:?}.", start_time.elapsed())).bold().green());
    

    let hash_progress_cb = |info | {
        match info {
            file_hashing::ProgressInfo::Yield(p) => {
                tracing::info!("Hashing... ({}%)", p);
            },
            file_hashing::ProgressInfo::Error(error) => {
                panic!("Failed to hash file: {:?}", error)
            }
        }
    };

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
             hash_progress_cb
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
