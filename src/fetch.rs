use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Duration,
};

use futures::{stream, StreamExt};
use humansize::{format_size, DECIMAL};
use regex::Regex;
use tokio::{fs::create_dir_all, time::Instant};
use tokio_retry::{strategy::FixedInterval, Retry};

use crate::{
    api::{sekai_client::SekaiClient, url::UrlProvider},
    config::{download_ab_config::DownloadAbConfig, fetch_config::FetchConfig},
    constants::{color, strings},
    crypto::assetbundle,
    error::CommandError,
    models::api::{Assetbundle, AssetbundleInfo},
    utils::{
        fs::{extract_suitemaster_file, write_file},
        progress::ProgressBar,
    },
    Error,
};

#[derive(Debug)]
struct AssetbundlePathArgs {
    asset_version: String,
    asset_hash: String,
    host_hash: String,
}

/// Responsible for fetching assets or information from the game's official servers.
pub struct Fetcher<P: UrlProvider> {
    config: FetchConfig<P>,
    client: SekaiClient<P>,
}

impl<P: UrlProvider> Fetcher<P> {
    /// Create a new Fetcher using the provided [`crate::config::fetch_config::FetchConfig`]
    pub async fn new(config: FetchConfig<P>) -> Result<Self, Error> {
        let client = SekaiClient::new_with_url_provider(
            config.version.clone(),
            config.hash.clone(),
            config.platform,
            config.aes_config,
            config.url_provider.clone(),
        )
        .await?;

        Ok(Self { config, client })
    }

    /// Gets assetbundle info from the game server.
    ///
    /// If asset_version or host_hash are not provided, their most recent values will be used.
    pub async fn get_ab_info(
        &mut self,
        asset_version: Option<String>,
        host_hash: Option<String>,
    ) -> Result<AssetbundleInfo, Error> {
        // get asset hash only if we got the most recent versions of the asset_version & host_hash
        let asset_hash = if asset_version.is_none() && host_hash.is_none() {
            let user_signup = self.client.user_signup().await?;
            let user_auth_response = self
                .client
                .user_login(
                    user_signup.user_registration.user_id,
                    user_signup.credential,
                )
                .await?;
            Some(user_auth_response.asset_hash)
        } else {
            None
        };

        // get the assetbundle host hash
        let host_hash = if let Some(host_hash) = host_hash {
            host_hash
        } else {
            self.client.get_game_version().await?.assetbundle_host_hash
        };

        // get system information
        let asset_version = if let Some(version) = asset_version {
            Ok(version)
        } else {
            let system_info = self.client.get_system().await?;
            if let Some(most_recent_version) = system_info.app_versions.last() {
                Ok(most_recent_version.asset_version.clone())
            } else {
                Err(CommandError::NotFound(
                    strings::command::error::NO_RECENT_VERSION.to_string(),
                ))
            }
        }?;

        // get the assetbundle info
        let mut assetbundle_info = self
            .client
            .get_assetbundle_info(&asset_version, &host_hash)
            .await?;
        assetbundle_info.host_hash = Some(host_hash.to_string());
        assetbundle_info.hash = asset_hash;

        Ok(assetbundle_info)
    }

    /// Downloads every available suitemasterfile to a specified ``out_path``.
    ///
    /// If ``out_path`` does not exist, it will be created.
    /// If this Fetcher was created using a configuration with ``decrypt`` set to true, the suitemaster files will be decrypted as .json files.
    ///
    /// Returns the number of suitemasterfiles that were successfully processed.
    pub async fn download_suite(&mut self, out_path: impl AsRef<Path>) -> Result<usize, Error> {
        let show_progress = !self.config.quiet;

        // create communication spinner
        let login_spinner = if show_progress {
            println!(
                "{}[1/2] {}{}",
                color::TEXT_VARIANT.render_fg(),
                color::TEXT.render_fg(),
                strings::command::COMMUNICATING,
            );
            Some(ProgressBar::spinner())
        } else {
            None
        };

        // see what suite master split files are available for download
        let user_signup = self.client.user_signup().await?;
        let user_login = self
            .client
            .user_login(
                user_signup.user_registration.user_id,
                user_signup.credential,
            )
            .await?;

        // clear login spinner if it exists
        if let Some(spinner) = login_spinner {
            spinner.finish_and_clear()
        }

        // create download progress bar
        let suitemaster_split_paths = user_login.suite_master_split_path;
        let split_count = suitemaster_split_paths.len();

        let download_progress = if show_progress {
            println!(
                "{}[2/2] {}{}",
                color::TEXT_VARIANT.render_fg(),
                color::TEXT.render_fg(),
                strings::command::DOWNLOADING,
            );
            println!(
                "{}{} {}{}",
                color::TEXT_VARIANT.render_fg(),
                strings::command::SUITE_VERSION,
                color::TEXT.render_fg(),
                user_login.data_version
            );

            let progress = ProgressBar::progress(split_count as u64);
            progress.enable_steady_tick(Duration::from_millis(100));
            Some(progress)
        } else {
            None
        };

        // download suite master split files
        let out_path = out_path.as_ref();
        let retry_strat = FixedInterval::from_millis(200).take(self.config.retry);
        let download_start = Instant::now();
        let do_decrypt = self.config.decrypt;

        let download_results: Vec<Result<(), CommandError>> =
            stream::iter(&suitemaster_split_paths)
                .map(|api_path| async {
                    let retry_result = Retry::spawn(retry_strat.clone(), || {
                        download_suitemasterfile(&self.client, api_path, out_path, do_decrypt)
                    })
                    .await;
                    if let Some(progress) = &download_progress {
                        progress.inc(1);
                    }
                    retry_result
                })
                .buffer_unordered(self.config.concurrency)
                .collect()
                .await;

        // stop progress bar
        if let Some(progress) = download_progress {
            progress.finish_and_clear();
        }

        // print result
        let success_count = download_results
            .iter()
            .filter(|&result| result.is_ok())
            .count();

        if show_progress {
            println!(
                "{}Successfully {} {} / {} files in {:?}{}",
                color::SUCCESS.render_fg(),
                strings::command::DOWNLOADED,
                success_count,
                split_count,
                Instant::now().duration_since(download_start),
                color::TEXT.render_fg(),
            );
        }

        Ok(success_count)
    }

    /// Downloads assetbundles to the provided ``out_dir`` using the provided config.
    pub async fn download_ab(
        &mut self,
        out_dir: impl AsRef<Path>,
        config: DownloadAbConfig,
    ) -> Result<usize, Error> {
        let show_progress = !self.config.quiet;

        // create assetbundle spinner
        let ab_info_spinner = if show_progress {
            println!(
                "{}[1/2] {}{}",
                color::TEXT_VARIANT.render_fg(),
                color::TEXT.render_fg(),
                strings::command::RETRIEVING_AB_INFO,
            );
            Some(ProgressBar::spinner())
        } else {
            None
        };

        // get assetbundle info
        let assetbundle_info = match config.info {
            None => {
                self.get_ab_info(config.asset_version, config.host_hash)
                    .await?
            }
            Some(info) => {
                if config.update {
                    let latest_info = self
                        .get_ab_info(config.asset_version, config.host_hash)
                        .await?;

                    AssetbundleInfo {
                        bundles: get_assetbundles_differences(latest_info.bundles, &info.bundles),
                        ..latest_info
                    }
                } else {
                    info
                }
            }
        };

        // stop assetbundle info spinner
        if let Some(spinner) = ab_info_spinner {
            spinner.finish_and_clear()
        }

        // extract data from assetbundle_info
        let ab_path_args = AssetbundlePathArgs {
            asset_version: assetbundle_info.version.clone(),
            asset_hash: assetbundle_info.hash.clone().unwrap_or_default(),
            host_hash: assetbundle_info.host_hash.clone().unwrap_or_default(),
        };

        // convert out_dir to a path.
        let out_dir = out_dir.as_ref();
        create_dir_all(out_dir).await?;

        // calculate out paths
        let mut total_bundle_size = 0;
        let mut to_download_bundles: Vec<(Assetbundle, PathBuf)> = Vec::new();

        let bundle_name_re = config
            .filter
            .as_ref()
            .and_then(|filter| Regex::new(filter).ok());

        for (bundle_name, bundle) in assetbundle_info.bundles {
            if bundle_name_re
                .as_ref()
                .map_or(true, |re| re.find(&bundle_name).is_some())
            {
                let out_path = out_dir.join(self.client.url_provider.assetbundle_path(
                    &ab_path_args.asset_version,
                    &ab_path_args.asset_hash,
                    &self.client.app.platform,
                    &bundle.bundle_name,
                ));

                total_bundle_size += bundle.file_size;
                to_download_bundles.push((bundle, out_path));
            }
        }

        if config.filter.is_some() && bundle_name_re.is_none() && show_progress {
            println!(
                "{}{}{}",
                color::ERROR.render_fg(),
                strings::command::INVALID_RE,
                color::TEXT.render_fg()
            )
        }

        // make sure the out_dir has enough space
        let available_space = fs2::available_space(out_dir)?;
        if (total_bundle_size > available_space) && show_progress {
            return Err(CommandError::NotEnoughSpace(format!(
                "this operation requires {} of free space. you only have {} available.",
                format_size(total_bundle_size, DECIMAL),
                format_size(available_space, DECIMAL)
            ))
            .into());
        }

        // create download progress bar
        let download_progress = if show_progress {
            println!(
                "{}[2/2] {}{}",
                color::TEXT_VARIANT.render_fg(),
                color::TEXT.render_fg(),
                strings::command::DOWNLOADING,
            );
            Some(ProgressBar::download(total_bundle_size))
        } else {
            None
        };

        // download bundles
        let download_start = Instant::now();
        let retry_strat = FixedInterval::from_millis(200).take(self.config.retry);
        let do_decrypt = self.config.decrypt;

        let download_results: Vec<Result<(), CommandError>> = stream::iter(&to_download_bundles)
            .map(|(bundle, out_path)| async {
                Retry::spawn(retry_strat.clone(), || {
                    download_bundle(
                        &self.client,
                        bundle,
                        out_path,
                        &ab_path_args,
                        &download_progress,
                        do_decrypt,
                    )
                })
                .await
            })
            .buffer_unordered(self.config.concurrency)
            .collect()
            .await;

        // count successes & print errors if debug is enabled
        let success_count = download_results
            .iter()
            .filter(|&result| {
                if let Err(err) = result {
                    if show_progress {
                        println!("assetbundle download error: {:?}", err);
                    }
                    false
                } else {
                    true
                }
            })
            .count();

        // stop progress bar & print the sucess message
        if let Some(progress) = download_progress {
            progress.finish_and_clear();
            println!(
                "{}Successfully {} {} / {} files in {:?}{}",
                color::SUCCESS.render_fg(),
                strings::command::DOWNLOADED,
                success_count,
                to_download_bundles.len(),
                Instant::now().duration_since(download_start),
                color::TEXT.render_fg(),
            );
        }

        Ok(success_count)
    }
}

/// Downloads a suitemasterfile at the provided path using the given SekaiClient.
///
/// This will unpack each suitemasterfile and save the contents to the provided out_path.
///
/// If encrypt is true, the suitemaster file will not be unpacked.
async fn download_suitemasterfile<P: UrlProvider>(
    client: &SekaiClient<P>,
    api_file_path: &str,
    out_path: &Path,
    decrypt: bool,
) -> Result<(), CommandError> {
    if decrypt {
        let value = client.get_suitemasterfile_as_value(api_file_path).await?;
        extract_suitemaster_file(value, out_path).await?;
        Ok(())
    } else {
        let file_bytes = client.get_suitemasterfile(api_file_path).await?;
        if let Some(file_name) = Path::new(api_file_path).file_name() {
            write_file(&out_path.join(file_name), &file_bytes).await?;
            Ok(())
        } else {
            Err(CommandError::NotFound(format!(
                "file name not found for api file path: {}",
                api_file_path
            )))
        }
    }
}

/// Compares two HashMaps of [crate::models::api::Assetbundle].
///
/// Returns a new HashMap of [crate::models::api::Assetbundle] where
/// a bundle in main_bundles exists in compare_bundles,
/// but has a different hash value.
fn get_assetbundles_differences(
    main_bundles: HashMap<String, Assetbundle>,
    compare_bundles: &HashMap<String, Assetbundle>,
) -> HashMap<String, Assetbundle> {
    main_bundles
        .into_iter()
        .filter(|(bundle_name, bundle)| {
            compare_bundles
                .get(bundle_name)
                .map_or(true, |compare_bundle| compare_bundle.hash != bundle.hash)
        })
        .collect()
}

/// Downloads an assetbundle to a provided path.
///
/// If decrypt is false, the downloaded assetbundle will remain encrypted.
async fn download_bundle<P: UrlProvider>(
    client: &SekaiClient<P>,
    bundle: &Assetbundle,
    out_path: &Path,
    path_args: &AssetbundlePathArgs,
    download_progress: &Option<indicatif::ProgressBar>,
    decrypt: bool,
) -> Result<(), CommandError> {
    // download
    let mut ab_data = client
        .get_assetbundle(
            &path_args.asset_version,
            &path_args.asset_hash,
            &path_args.host_hash,
            &bundle.bundle_name,
        )
        .await?;

    // decrypt if desired
    if decrypt {
        assetbundle::decrypt_in_place(&mut ab_data).await?;
    }

    // write file
    write_file(out_path, &ab_data).await?;

    // increment progress
    if let Some(progress) = download_progress {
        progress.inc(bundle.file_size);
    }
    Ok(())
}
