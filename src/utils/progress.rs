use std::time::Duration;

use indicatif::ProgressStyle;

/// Convenience struct for generating [`indicatif::ProgressBar`] instances with twintail styling.
pub struct ProgressBar;

impl ProgressBar {
    /// Create a normal progress bar.
    pub fn progress(size: u64) -> indicatif::ProgressBar {
        indicatif::ProgressBar::new(size).with_style(
            ProgressStyle::with_template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len}")
                .unwrap_or(ProgressStyle::default_bar())
                .progress_chars("#-"),
        )
    }

    /// Create a progress bar that shows download progress, download speed, and ETA.
    pub fn download(size: u64) -> indicatif::ProgressBar {
        indicatif::ProgressBar::new(size).with_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
            )
            .unwrap_or(ProgressStyle::default_bar())
            .progress_chars("#-"),
        )
    }

    /// Create a new progress spinner that automatically ticks every 100ms.
    pub fn spinner() -> indicatif::ProgressBar {
        let spinner = indicatif::ProgressBar::new_spinner();
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner
    }
}
