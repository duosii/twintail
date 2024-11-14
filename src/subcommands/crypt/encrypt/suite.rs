use crate::constants::{color, strings};
use crate::crypto::aes_msgpack;
use crate::models::enums::Server;
use crate::utils::progress::{ProgressBar, WithProgress};
use crate::{error::CommandError, utils::fs::scan_path};
use clap::Args;
use futures::{stream, StreamExt};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use tokio::fs::create_dir_all;
use tokio::io::AsyncWriteExt;
use tokio::time::Instant;

#[derive(Debug, Args)]
pub struct EncryptSuiteArgs {
    /// If the input is a directory, whether to recursively decrypt valid files in that directory
    #[arg(long, short, default_value_t = false)]
    pub recursive: bool,

    /// The maximum number of files to encrypt simultaneously
    #[arg(long, short, default_value_t = crate::utils::available_parallelism())]
    pub concurrent: usize,

    /// The server to encrypt the suitemasterfiles for
    #[arg(short, long, value_enum, default_value_t = Server::Japan)]
    pub server: Server,

    /// Path to the file or directory to encrypt
    pub in_path: String,

    /// Path to a directory or file to output to.
    pub out_path: String,
}

/// Deserializes a json file into a serde_json Value.
async fn deserialize_file(path: &PathBuf) -> Result<(String, Value), CommandError> {
    let in_file = std::fs::File::open(path)?;

    // get file name
    let file_stem = path
        .file_stem()
        .and_then(|os_str| os_str.to_str())
        .ok_or_else(|| CommandError::NotFound("file name not found for file".to_string()))?;

    // deserialize contents
    let reader = std::io::BufReader::new(in_file);
    let deserialized: Value = serde_json::from_reader(reader)?;

    Ok((file_stem.to_string(), deserialized))
}

pub async fn encrypt_suite(args: EncryptSuiteArgs) -> Result<(), CommandError> {
    let in_path = PathBuf::from(args.in_path);

    // get paths that we need to encrypt
    let encrypt_start = Instant::now();
    let to_encrypt_paths = scan_path(&in_path, args.recursive).await?;

    println!(
        "{}[1/2] {}{}",
        color::TEXT_VARIANT.render_fg(),
        color::TEXT.render_fg(),
        strings::command::SUITE_PROCESSING,
    );
    let deserialize_progress = ProgressBar::progress(to_encrypt_paths.len() as u64);

    // deserialize all paths to [`serde_json::Value`]s.
    let deserialize_results: Vec<Result<(String, Value), CommandError>> =
        stream::iter(&to_encrypt_paths)
            .map(|path| deserialize_file(path).with_progress(&deserialize_progress))
            .buffer_unordered(args.concurrent)
            .collect()
            .await;

    deserialize_progress.finish();

    // merge all values into a map
    println!(
        "{}[2/2] {}{}",
        color::TEXT_VARIANT.render_fg(),
        color::TEXT.render_fg(),
        strings::command::SUITE_SAVING,
    );

    let mut values_map: Map<String, Value> = Map::new();
    for result in deserialize_results {
        match result {
            Ok((key, value)) => {
                values_map.insert(key, value);
            }
            Err(err) => {
                println!(
                    "{}{}{}{}",
                    color::clap::ERROR.render_fg(),
                    strings::command::error::SUITE_DESERIALIZE_ERROR,
                    err,
                    color::TEXT.render_fg()
                );
            }
        }
    }

    // serialize as msgpack
    let serialized = aes_msgpack::into_vec(&values_map, &args.server)?;

    // write to out directory
    let out_path = Path::new(&args.out_path).join(strings::command::SUITE_ENCRYPTED_FILE_NAME);
    if let Some(parent) = out_path.parent() {
        create_dir_all(parent).await?;
    }
    let mut out_file = tokio::fs::File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(out_path)
        .await?;
    out_file.write_all(&serialized).await?;

    // print the result
    println!(
        "{}Successfully {} {} files in {:?}.{}",
        color::SUCCESS.render_fg(),
        strings::crypto::encrypt::PROCESSED,
        values_map.len(),
        Instant::now().duration_since(encrypt_start),
        color::TEXT.render_fg(),
    );

    Ok(())
}
