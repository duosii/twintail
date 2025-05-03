use std::time::Duration;

use clap::Args;
use indicatif::ProgressBar;
use twintail_common::color;
use twintail_core::apk_extractor::ApkExtractor;

use crate::{Error, strings};

#[derive(Debug, Args)]
pub struct AppInfoArgs {
    /// Path to the APK to extract the app hash & version from
    pub apk_path: String,
}

/// Extracts app hash and version from an APK.
///
/// If successful, the hash and app version
/// will be printed out to the console.
pub fn app_info(args: AppInfoArgs) -> Result<(), Error> {
    // create assetbundle spinner
    println!(
        "{}[1/1] {}{}",
        color::TEXT_VARIANT.render_fg(),
        color::TEXT.render_fg(),
        strings::command::EXTRACTING,
    );
    let extract_spinner = ProgressBar::new_spinner();
    extract_spinner.enable_steady_tick(Duration::from_millis(100));

    // create extractor & extract app info
    let mut extractor = ApkExtractor::from_file(&args.apk_path)?;
    let app_info = extractor.extract()?;

    extract_spinner.finish_and_clear();

    // output results
    if app_info.hashes.is_empty() {
        println!(
            "{}{}{}",
            color::ERROR.render_fg(),
            strings::command::EXTRACT_FAIL,
            color::TEXT.render_fg(),
        );
    } else {
        println!(
            "{}{}{}",
            color::SUCCESS.render_fg(),
            strings::command::EXTRACT_SUCCESS,
            color::TEXT.render_fg(),
        );
        println!(
            "{}{} {}{}",
            color::TEXT_VARIANT.render_fg(),
            strings::command::EXTRACT_VERSION,
            color::TEXT.render_fg(),
            app_info
                .version
                .unwrap_or(strings::command::EXTRACT_MISSING.to_string())
        );

        for hash in app_info.hashes {
            println!(
                "{}{} {}{}",
                color::TEXT_VARIANT.render_fg(),
                strings::command::EXTRACT_HASH,
                color::TEXT.render_fg(),
                hash
            );
        }
    }

    Ok(())
}
