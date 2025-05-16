use clap::Args;
use std::io::{Write, stdin, stdout};
use tokio::{sync::watch::Receiver, time::Instant};
use twintail_common::{models::enums::Server, utils::progress::ProgressBar};
use twintail_core::{
    config::fetch_config::FetchConfig,
    fetch::{FetchState, Fetcher, GetUserInheritState, WriteUserSaveDataState},
};

use crate::{Error, color, strings};

#[derive(Debug, Args)]
pub struct SaveArgs {
    /// The current version of the app where the target account is located
    #[arg(short, long)]
    pub version: String,

    /// The current hash of the app where the target account is located
    #[arg(long)]
    pub hash: String,

    /// The inherit ID that the game generated for you when initiating the account transfer
    #[arg(long)]
    pub id: String,

    /// The password you used when initiating the account transfer
    #[arg(long, short)]
    pub password: String,

    /// The server to download the save from
    #[arg(short, long, value_enum, default_value_t = Server::Japan)]
    pub server: Server,

    /// Whether to output status messages
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// Automatically accept any confirmation prompts
    #[arg(short, long, default_value_t = false)]
    pub yes: bool,

    /// Whether to save suitemaster .json files in a more compact format, reducing their file size
    #[arg(long, default_value_t = false)]
    pub compact: bool,

    /// The directory to output the save data to
    pub out_path: Option<String>,
}

/// Watches a [`tokio::sync::watch::Receiver`] for state changes.
///
/// Prints information related to the progress of a save fetch.
async fn watch_fetch_save_state(mut receiver: Receiver<FetchState>) {
    let mut progress_bar: Option<indicatif::ProgressBar> = None;
    while receiver.changed().await.is_ok() {
        match *receiver.borrow_and_update() {
            FetchState::GetUserInherit(GetUserInheritState::GetInherit) => {
                println!(
                    "{}{}{}",
                    color::TEXT_VARIANT.render_fg(),
                    color::TEXT.render_fg(),
                    strings::command::INHERIT_GETTING_USER_DATA,
                );
                progress_bar = Some(ProgressBar::spinner())
            }
            FetchState::GetUserInherit(GetUserInheritState::Finish) => {
                if let Some(progress) = &progress_bar {
                    progress.finish_and_clear();
                }
            }
            FetchState::WriteUserSaveData(WriteUserSaveDataState::Login) => {
                if let Some(spinner) = &progress_bar {
                    spinner.finish_and_clear();
                }

                println!(
                    "{}{}{}",
                    color::TEXT_VARIANT.render_fg(),
                    color::TEXT.render_fg(),
                    strings::command::INHERIT_LOGGING_IN,
                );
                progress_bar = Some(ProgressBar::spinner())
            }
            FetchState::WriteUserSaveData(WriteUserSaveDataState::GetSaveData) => {
                if let Some(spinner) = &progress_bar {
                    spinner.finish_and_clear();
                }

                println!(
                    "{}{}{}",
                    color::TEXT_VARIANT.render_fg(),
                    color::TEXT.render_fg(),
                    strings::command::INHERIT_GETTING_SAVE_DATA,
                );
                progress_bar = Some(ProgressBar::spinner())
            }
            FetchState::WriteUserSaveData(WriteUserSaveDataState::Finish) => {
                if let Some(progress) = &progress_bar {
                    progress.finish_and_clear();
                }
                break;
            }
            _ => {}
        }
    }
}

pub async fn fetch_save(args: SaveArgs) -> Result<(), Error> {
    let show_progress = !args.quiet;

    // create fetcher
    let fetch_config = FetchConfig::builder(args.version, args.hash)
        .server(args.server)
        .pretty_json(!args.compact)
        .build();
    let (mut fetcher, state_recv) = Fetcher::new(fetch_config).await?;

    let state_watcher = if show_progress {
        Some(tokio::spawn(watch_fetch_save_state(state_recv)))
    } else {
        None
    };

    // fetch user inherit data
    let user_inherit = fetcher
        .get_user_inherit(&args.id, &args.password, false)
        .await?;

    if show_progress {
        println!();

        println!(
            "{}{}{}",
            color::TEXT_VARIANT.render_fg(),
            strings::command::INHERIT_USER_DETAILS,
            color::TEXT.render_fg(),
        );
        println!(
            "   {}{} {}{}",
            color::TEXT_VARIANT.render_fg(),
            strings::command::INHERIT_USER_ID,
            color::TEXT.render_fg(),
            user_inherit.after_user_gamedata.user_id
        );
        println!(
            "   {}{} {}{}",
            color::TEXT_VARIANT.render_fg(),
            strings::command::INHERIT_USER_NAME,
            color::TEXT.render_fg(),
            user_inherit.after_user_gamedata.name
        );
        println!(
            "   {}{} {}{}",
            color::TEXT_VARIANT.render_fg(),
            strings::command::INHERIT_USER_RANK,
            color::TEXT.render_fg(),
            user_inherit.after_user_gamedata.rank
        );
        println!();
    }

    if !args.yes {
        print!(
            "{}{}{}",
            color::WARNING.render_fg(),
            strings::command::INHERIT_CONTINUE_CONFIRM,
            color::TEXT.render_fg()
        );
        stdout().flush()?;

        // read confirmation response
        let mut response = String::new();
        stdin().read_line(&mut response)?;

        match response.to_lowercase().trim() {
            "y" => {}
            _ => {
                println!(
                    "{}{}{}",
                    color::ERROR.render_fg(),
                    strings::command::INHERIT_CANCELLED,
                    color::TEXT.render_fg()
                );
                return Ok(());
            }
        }

        println!();
    }

    // actually inherit the account to get the login credential
    let user_inherit = fetcher
        .get_user_inherit(&args.id, &args.password, true)
        .await?;

    let credential = user_inherit.credential.unwrap_or_default();
    if credential.is_empty() {
        if show_progress {
            println!(
                "{}{}{}",
                color::ERROR.render_fg(),
                strings::command::INHERIT_NO_CREDENTIAL,
                color::TEXT.render_fg()
            );
        }
        return Ok(());
    }

    // write save data
    let write_start = Instant::now();
    let out_path = args.out_path.unwrap_or_default();
    fetcher
        .write_user_save_data(
            user_inherit.after_user_gamedata.user_id,
            credential,
            &out_path,
        )
        .await?;

    if let Some(watcher) = state_watcher {
        watcher.await?;
        println!();
        println!(
            "✅ {}Save data written to '{}' in {:?}. {}",
            color::SUCCESS.render_fg(),
            out_path,
            Instant::now().duration_since(write_start),
            color::TEXT.render_fg()
        );
        println!();
        println!(
            "⚠️ {}{}{}",
            color::WARNING.render_fg(),
            strings::command::INHERIT_FINISH_WARNING,
            color::TEXT.render_fg()
        );
    }

    Ok(())
}
