pub mod decrypt;
pub mod encrypt;

use crate::{
    constants::color,
    crypto::assetbundle::{crypt_file, CryptOperation},
    error::{AssetbundleError, CommandError},
    utils::{fs::scan_directory, progress::WithProgress},
};
use futures::{stream, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::{path::PathBuf, time::Duration};
use tokio::time::Instant;

pub struct CryptStrings<'a> {
    pub process: &'a str,
    pub processed: &'a str,
}

pub struct CryptArgs<'a> {
    in_path: &'a str,
    recursive: bool,
    concurrent: usize,
    operation: CryptOperation,
    strings: CryptStrings<'a>,
    out_path: &'a Option<String>,
}

/// Decrypts or encrypts a file/directory that is/contains an assetbundle.
pub async fn crypt_assetbundle<'a>(args: CryptArgs<'a>) -> Result<(), CommandError> {
    let in_path = PathBuf::from(args.in_path);
    let out_path = if let Some(out) = args.out_path {
        PathBuf::from(out)
    } else {
        in_path.clone()
    };
    let in_place = in_path == out_path;

    // get the paths that we need to decrypt
    println!(
        "{}[1/2] {}Scanning files...",
        color::TEXT_VARIANT.render_fg(),
        color::TEXT.render_fg()
    );
    let scan_progress_bar = ProgressBar::new_spinner();
    scan_progress_bar.enable_steady_tick(Duration::from_millis(100));

    let in_paths = if in_path.is_dir() {
        scan_directory(in_path.clone(), args.recursive).await?
    } else {
        vec![in_path.clone()]
    };
    scan_progress_bar.finish_and_clear();

    // start the processing step
    let decrypt_start = Instant::now();
    println!(
        "{}[2/2] {}{} files...",
        color::TEXT_VARIANT.render_fg(),
        color::TEXT.render_fg(),
        args.strings.process,
    );

    // compute paths
    let in_out_paths: Vec<(PathBuf, PathBuf)> = in_paths
        .into_iter()
        .map(|path| {
            if in_place {
                (path.clone(), path)
            } else {
                let relative = path.strip_prefix(&in_path).ok().unwrap_or(&path);
                let out = out_path.join(relative);
                (path, out)
            }
        })
        .collect();

    // asynchronously encrypt/decrypt the files
    let total_path_count: u64 = in_out_paths.len() as u64;
    let progress_bar = ProgressBar::new(total_path_count).with_style(
        ProgressStyle::with_template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len}")
            .unwrap_or(ProgressStyle::default_bar())
            .progress_chars("#-"),
    );
    let decrypt_result: Vec<Result<(), AssetbundleError>> = stream::iter(&in_out_paths)
        .map(|paths| crypt_file(&paths.0, &paths.1, &args.operation).with_progress(&progress_bar))
        .buffer_unordered(args.concurrent)
        .collect()
        .await;
    let success_count = decrypt_result
        .iter()
        .filter(|&result| result.is_ok())
        .count();

    // stop progress bar & print the sucess message
    progress_bar.finish_and_clear();
    println!(
        "{}Successfully {} {} / {} files in {:?}.{}",
        color::SUCCESS.render_fg(),
        args.strings.processed,
        success_count,
        total_path_count,
        Instant::now().duration_since(decrypt_start),
        color::TEXT.render_fg(),
    );

    Ok(())
}
