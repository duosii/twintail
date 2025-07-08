use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use futures::{StreamExt, stream};
use humansize::{DECIMAL, format_size};
use regex::Regex;
use tokio::{fs::create_dir_all, sync::watch};
use tokio_retry::{Retry, strategy::FixedInterval};
use twintail_common::models::OptionalBuilder;
use twintail_sekai::{
    models::{Assetbundle, AssetbundleInfo, UserInherit},
    sekai_client::{SekaiClient, SekaiClientBuilder},
    url::UrlProvider,
};

use crate::{
    Error,
    config::{download_ab_config::DownloadAbConfig, fetch_config::FetchConfig},
    crypto::assetbundle,
    fs::{extract_suitemaster_file, write_file},
};

#[derive(Clone, Copy)]
pub enum DownloadSuiteState {
    /// The suite downloader is communicating with the game server
    Communicate,
    /// The specified number of suite files are being downloaded
    DownloadStart(usize),
    /// A suite file was downloaded
    FileDownload,
    /// The suite download finished. Contains the number of files that were downloaded and the number of files that were available to download
    Finish,
}

#[derive(Clone, Copy)]
pub enum DownloadAbState {
    /// assetbundle info is being retrieved from the game server
    RetrieveAbInfo,
    /// an invalid regular expression was given to the downloader
    InvalidRegEx,
    /// the given number of bytes are being downloaded
    DownloadStart(u64),
    /// a file of the provided size in bytes was downloaded
    FileDownload(u64),
    /// the download process finished
    Finish,
}

#[derive(Clone, Copy)]
pub enum GetUserInheritState {
    /// communicating with the game server to get inherit data
    GetInherit,
    /// user inherit data has been received
    Finish,
}

#[derive(Clone, Copy)]
pub enum WriteUserSaveDataState {
    /// logging into the user's account
    Login,
    /// get the user's save data
    GetSaveData,
    /// save data has been written to the specified location
    Finish,
}

#[derive(Clone, Copy)]
pub enum FetchState {
    NoState,
    DownloadSuite(DownloadSuiteState),
    DownloadAb(DownloadAbState),
    GetUserInherit(GetUserInheritState),
    WriteUserSaveData(WriteUserSaveDataState),
}

#[derive(Debug)]
struct AssetbundlePathArgs {
    asset_version: String,
    asset_hash: String,
    host_hash: String,
}

/// Responsible for fetching assets or information from the game's official servers.
pub struct Fetcher<P: UrlProvider> {
    state_sender: watch::Sender<FetchState>,
    config: FetchConfig<P>,
    client: SekaiClient<P>,
}

impl<P: UrlProvider> Fetcher<P> {
    /// Create a new Fetcher using the provided [`crate::config::fetch_config::FetchConfig`]
    pub async fn new(config: FetchConfig<P>) -> Result<(Self, watch::Receiver<FetchState>), Error> {
        let client = SekaiClientBuilder::new(
            config.aes_config.clone(),
            config.jwt_key.clone(),
            config.platform,
            config.url_provider.clone(),
        )
        .map(config.hash.clone(), |builder, hash| builder.app_hash(hash))
        .map(config.version.clone(), |builder, hash| {
            builder.app_version(hash)
        })
        .build()
        .await?;

        let (state_sender, recv) = watch::channel(FetchState::NoState);

        Ok((
            Self {
                state_sender,
                config,
                client,
            },
            recv,
        ))
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
        let asset_hash = {
            let user_signup = self.client.user_signup().await?;
            let user_auth_response = self
                .client
                .user_login(
                    user_signup.user_registration.user_id,
                    user_signup.credential,
                )
                .await?;
            user_auth_response.asset_hash
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
                Err(Error::NotFound("most recent game version not found".into()))
            }
        }?;

        // get the assetbundle info
        let mut assetbundle_info = self
            .client
            .get_assetbundle_info(&asset_version, &asset_hash, &host_hash)
            .await?;
        assetbundle_info.host_hash = Some(host_hash.to_string());
        assetbundle_info.hash = Some(asset_hash);

        Ok(assetbundle_info)
    }

    /// Downloads every available suitemasterfile to a specified ``out_path``.
    ///
    /// If ``out_path`` does not exist, it will be created.
    /// If this Fetcher was created using a configuration with ``decrypt`` set to true, the suitemaster files will be decrypted as .json files.
    ///
    /// Returns:
    /// - The number of suitemasterfiles that were successfully processed
    /// - The total number of suitemasterfiles that were available to download,
    /// - The current suitemaster data version
    pub async fn download_suite(
        &mut self,
        out_path: impl AsRef<Path>,
    ) -> Result<(usize, usize, String), Error> {
        // see what suite master split files are available for download
        self.state_sender
            .send_replace(FetchState::DownloadSuite(DownloadSuiteState::Communicate));

        let user_signup = self.client.user_signup().await?;
        let user_login = self
            .client
            .user_login(
                user_signup.user_registration.user_id,
                user_signup.credential,
            )
            .await?;

        // create download progress bar
        let suitemaster_split_paths = user_login.suite_master_split_path;
        let split_count = suitemaster_split_paths.len();

        self.state_sender.send_replace(FetchState::DownloadSuite(
            DownloadSuiteState::DownloadStart(split_count),
        ));

        // download suite master split files
        let out_path = out_path.as_ref();
        let retry_strat = FixedInterval::from_millis(200).take(self.config.retry);
        let do_decrypt = self.config.decrypt;
        let pretty_json = self.config.pretty_json;

        let download_results: Vec<Result<(), Error>> = stream::iter(&suitemaster_split_paths)
            .map(|api_path| async {
                let retry_result = Retry::spawn(retry_strat.clone(), || {
                    download_suitemasterfile(
                        &self.client,
                        api_path,
                        out_path,
                        do_decrypt,
                        pretty_json,
                    )
                })
                .await;
                self.state_sender
                    .send_replace(FetchState::DownloadSuite(DownloadSuiteState::FileDownload));
                retry_result
            })
            .buffer_unordered(self.config.concurrency)
            .collect()
            .await;

        // print result
        let success_count = download_results
            .iter()
            .filter(|&result| result.is_ok())
            .count();

        self.state_sender
            .send_replace(FetchState::DownloadSuite(DownloadSuiteState::Finish));

        Ok((success_count, split_count, user_login.data_version))
    }

    /// Downloads assetbundles to the provided ``out_dir`` using the provided config.
    ///
    /// Returns:
    /// - the number of files that were successfully downloaded
    /// - the number of files that were available for download
    /// - a Vec of errors that ocurred when downloading specific files
    pub async fn download_ab(
        &mut self,
        out_dir: impl AsRef<Path>,
        config: DownloadAbConfig,
    ) -> Result<(usize, usize, Vec<Error>), Error> {
        // create assetbundle spinner
        self.state_sender
            .send_replace(FetchState::DownloadAb(DownloadAbState::RetrieveAbInfo));

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
                .is_none_or(|re| re.find(&bundle_name).is_some())
            {
                let out_path = out_dir.join(self.client.url_provider.assetbundle_path(
                    &ab_path_args.asset_version,
                    &ab_path_args.asset_hash,
                    &self.client.platform,
                    &bundle.bundle_name,
                ));

                total_bundle_size += bundle.file_size;
                to_download_bundles.push((bundle, out_path));
            }
        }

        if config.filter.is_some() && bundle_name_re.is_none() {
            self.state_sender
                .send_replace(FetchState::DownloadAb(DownloadAbState::InvalidRegEx));
        }

        // make sure the out_dir has enough space
        let available_space = fs2::available_space(out_dir)?;
        if total_bundle_size > available_space {
            return Err(Error::NotEnoughSpace(format!(
                "this operation requires {} of free space. you only have {} available.",
                format_size(total_bundle_size, DECIMAL),
                format_size(available_space, DECIMAL)
            )));
        }

        // create download progress bar
        self.state_sender
            .send_replace(FetchState::DownloadAb(DownloadAbState::DownloadStart(
                total_bundle_size,
            )));

        // download bundles
        let retry_strat = FixedInterval::from_millis(200).take(self.config.retry);
        let do_decrypt = self.config.decrypt;

        let download_results: Vec<Result<(), Error>> = stream::iter(&to_download_bundles)
            .map(|(bundle, out_path)| async {
                let download_result = Retry::spawn(retry_strat.clone(), || {
                    download_bundle(&self.client, bundle, out_path, &ab_path_args, do_decrypt)
                })
                .await;
                if download_result.is_ok() {
                    self.state_sender.send_replace(FetchState::DownloadAb(
                        DownloadAbState::FileDownload(bundle.file_size),
                    ));
                }
                download_result
            })
            .buffer_unordered(self.config.concurrency)
            .collect()
            .await;

        // count successes & print errors if debug is enabled
        let download_errors: Vec<_> = download_results
            .into_iter()
            .filter_map(|result| result.err())
            .collect();

        // stop progress bar & print the sucess message
        self.state_sender
            .send_replace(FetchState::DownloadAb(DownloadAbState::Finish));

        let total_bundle_count = to_download_bundles.len();
        Ok((
            total_bundle_count - download_errors.len(),
            total_bundle_count,
            download_errors,
        ))
    }

    /// Performs a request to get a user's account inherit details.
    ///
    /// If execute is true, the account will be inherited and the returned UserInherit will contain an authentication credential JWT.
    ///
    /// This credential is used for performing authenticated requests.
    pub async fn get_user_inherit(
        &self,
        inherit_id: &str,
        password: &str,
        execute: bool,
    ) -> Result<UserInherit, Error> {
        self.state_sender
            .send_replace(FetchState::GetUserInherit(GetUserInheritState::GetInherit));

        let user_inherit = self
            .client
            .get_user_inherit(inherit_id, password, execute)
            .await?;

        self.state_sender
            .send_replace(FetchState::GetUserInherit(GetUserInheritState::Finish));

        Ok(user_inherit)
    }

    /// Gets a user's save data, provided with their user_id and login credential.
    ///
    /// If successful, a PathBuf representing where the save data was written will be returned.
    pub async fn write_user_save_data(
        &mut self,
        user_id: usize,
        credential: String,
        out_dir: impl AsRef<Path>,
    ) -> Result<PathBuf, Error> {
        self.state_sender
            .send_replace(FetchState::WriteUserSaveData(WriteUserSaveDataState::Login));
        self.client.user_login(user_id, credential).await?;
        self.state_sender
            .send_replace(FetchState::WriteUserSaveData(
                WriteUserSaveDataState::GetSaveData,
            ));

        // convert retrieve save data & convert to json
        let save_data = self.client.get_user_suite(user_id).await?;
        let json_save_data = if self.config.pretty_json {
            serde_json::to_vec_pretty(&save_data)
        } else {
            serde_json::to_vec(&save_data)
        }?;

        // save the save data
        let out_path = out_dir.as_ref().join(format!("{}.json", user_id));
        write_file(&out_path, &json_save_data).await?;

        self.state_sender
            .send_replace(FetchState::WriteUserSaveData(
                WriteUserSaveDataState::Finish,
            ));

        Ok(out_path)
    }
}

/// Downloads a suitemasterfile at the provided path using the given SekaiClient.
///
/// This will unpack each suitemasterfile and save the contents to the provided out_path.
///
/// If decrypt is false, the suitemaster file will not be unpacked.
///
/// If pretty is true, the extacted suitemaster files will be saved in a more readable format.
async fn download_suitemasterfile<P: UrlProvider>(
    client: &SekaiClient<P>,
    api_file_path: &str,
    out_path: &Path,
    decrypt: bool,
    pretty: bool,
) -> Result<(), Error> {
    if decrypt {
        let value = client.get_suitemasterfile_as_value(api_file_path).await?;
        extract_suitemaster_file(value, out_path, pretty).await?;
        Ok(())
    } else {
        let file_bytes = client.get_suitemasterfile(api_file_path).await?;
        if let Some(file_name) = Path::new(api_file_path).file_name() {
            write_file(&out_path.join(file_name), &file_bytes).await?;
            Ok(())
        } else {
            Err(Error::NotFound(format!(
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
                .is_none_or(|compare_bundle| compare_bundle.hash != bundle.hash)
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
    decrypt: bool,
) -> Result<(), Error> {
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
        // if the asset bundle wasn't encrypted, just ignore the error since it's already decrypted.
        match assetbundle::decrypt_in_place(&mut ab_data).await {
            Ok(_) => Ok(()),
            Err(Error::NotEncrypted) => Ok(()),
            Err(err) => Err(err),
        }?;
    }

    // write file
    write_file(out_path, &ab_data).await?;
    Ok(())
}
