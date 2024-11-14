use crate::{
    constants::{color, strings},
    crypto::aes_msgpack,
    error::CommandError,
    models::enums::Server,
    utils::{
        fs::{extract_suitemaster_file, scan_path},
        progress::{ProgressBar, WithProgress},
    },
};
use clap::Args;
use futures::{stream, StreamExt};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::{fs::File, io::AsyncReadExt, time::Instant};

#[derive(Debug, Args)]
pub struct DecryptSuiteArgs {
    /// If the input is a directory, whether to recursively decrypt valid files in that directory
    #[arg(long, short, default_value_t = false)]
    pub recursive: bool,

    /// The maximum number of files to decrypt simultaneously
    #[arg(long, short, default_value_t = crate::utils::available_parallelism())]
    pub concurrent: usize,

    /// The server to decrypt the suitemasterfiles for
    #[arg(short, long, value_enum, default_value_t = Server::Japan)]
    pub server: Server,

    /// Whether to output debug messages, such as errors
    #[arg(long, short, default_value_t = false)]
    pub debug: bool,

    /// Path to the file or directory to decrypt
    pub in_path: String,

    /// Path to a directory or file to output to.
    pub out_path: String,
}

/// Reads the file at the input path as a [`serde_json::Value`]
/// and extracts its inner fields to out_path as .json files.
async fn decrypt_suitemaster_file(
    in_path: PathBuf,
    out_path: &Path,
    server: &Server,
) -> Result<(), CommandError> {
    // read in file
    let mut file = File::open(in_path).await?;
    let mut file_buf = Vec::new();
    file.read_to_end(&mut file_buf).await?;

    // deserialize as a value
    let deserialized: Value = aes_msgpack::from_slice(&file_buf, server)?;

    // write to out_path
    extract_suitemaster_file(deserialized, out_path).await?;

    Ok(())
}

/// Decrypts encrypted suitemaster files into individual .json files.
pub async fn decrypt_suite(args: DecryptSuiteArgs) -> Result<(), CommandError> {
    // get paths that we need to decrypt
    let decrypt_start = Instant::now();
    let to_decrypt_paths = scan_path(&Path::new(&args.in_path), args.recursive).await?;
    let out_path = Path::new(&args.out_path);

    // create decrypt progress bar
    println!(
        "{}[1/1] {}{}",
        color::TEXT_VARIANT.render_fg(),
        color::TEXT.render_fg(),
        strings::command::SUITE_DECRYPTING,
    );
    let decrypt_progress = ProgressBar::progress(to_decrypt_paths.len() as u64);

    // begin decrypting
    let server = args.server;
    let decrypt_results: Vec<Result<(), CommandError>> = stream::iter(to_decrypt_paths)
        .map(|in_path| {
            decrypt_suitemaster_file(in_path, &out_path, &server).with_progress(&decrypt_progress)
        })
        .buffer_unordered(args.concurrent)
        .collect()
        .await;

    decrypt_progress.finish_and_clear();

    // count the number of successes
    let do_debug = args.debug;
    let success_count = decrypt_results
        .iter()
        .filter(|&result| {
            if let Err(err) = result {
                if do_debug {
                    println!("suite decrypt error: {:?}", err);
                }
                false
            } else {
                true
            }
        })
        .count();

    // print the result
    println!(
        "{}Successfully {} {} files in {:?}.{}",
        color::SUCCESS.render_fg(),
        strings::crypto::decrypt::PROCESSED,
        success_count,
        Instant::now().duration_since(decrypt_start),
        color::TEXT.render_fg(),
    );

    Ok(())
}
