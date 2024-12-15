use clap::Parser;
use console::style;
use eyre::Result;
use std::{path::Path, time::Instant};
mod args;
use args::Args;
mod file_ops;
mod hash_ops;
mod log;
mod path_ops;
mod progress;

fn main() -> Result<()> {
    let start_time = Instant::now();
    log::init_logger();

    let args = Args::parse();

    if !args.no_keep_awake {
        // Stopped when program ends
        keepawake::Builder::default()
            .app_name(env!("CARGO_PKG_NAME"))
            .display(args.keep_display_awake)
            .idle(true);
    }

    let destination_path = path_ops::ensure_valid_paths(&args)?;

    for source in &args.source {
        let source_path = Path::new(&source);

        let mut current_destination = destination_path.clone();

        if source_path.is_file() && destination_path.is_dir() {
            current_destination = current_destination.join(source_path.file_name().unwrap());
        }
        let destination_path = current_destination.as_path();
        tracing::debug!("destination: {}", destination_path.display());

        let pb = progress::create_progress_bar();

        // Perform the copy operation
        println!(
            "{}",
            style(format!(
                "Copy {} to {}",
                source_path.display(),
                destination_path.display()
            ))
            .bold()
            .dim(),
        );
        file_ops::perform_copy_operation(&args, source_path, destination_path, &pb).unwrap();

        println!(
            "{}",
            style(format!(
                "Copy completed successfully in {:.2?}.",
                start_time.elapsed()
            ))
            .bold()
            .green()
        );

        if args.verify {
            tracing::info!("Computing hash...");
            hash_ops::verify_hash(source_path, destination_path)?;
        }
    }
    Ok(())
}
