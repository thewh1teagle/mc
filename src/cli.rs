use clap::Parser;

/// Command-line utility for copying files or directories with optional recursion and overwriting.
#[derive(Parser, Debug)]
#[command(about = "Copies files or directories with options for recursion and overwriting.")]
pub struct Args {
    /// Source file or directory to copy
    #[arg(required = true)]
    pub source: Vec<String>,

    /// Destination file or directory
    #[arg(required = true)]
    pub destination: String,

    /// Overwrite destination if it exists
    #[arg(short, long)]
    pub force: bool,

    /// Hard link file
    #[arg(long)]
    pub hard_link: bool,

    /// Symbol link file
    #[arg(long)]
    pub symlink: bool,

    /// Verify hash of folder / file once copied
    #[arg(long)]
    pub verify: bool,

    /// Disable progress bar
    #[arg(long)]
    pub no_progress: bool,

    /// Disable keep system awake while copy
    #[arg(long)]
    pub no_keep_awake: bool,

    /// Keep display awake while copy
    #[arg(long)]
    pub keep_display_awake: bool,
}
