use eyre::Result;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::fmt::Write;

pub fn create_progress_bar() -> Result<Option<ProgressBar>> {
    Ok(Some(
        ProgressBar::new(0)
            .with_style(
                ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta_precise})")
                    ?
                    .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
                        write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
                    })
                    .progress_chars("#>-")
            )
    ))
}
