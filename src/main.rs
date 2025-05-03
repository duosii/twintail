#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let cli_result = twintail_cli::run().await;

    if let Err(err) = cli_result {
        err.print()?;
    }

    Ok(())
}
