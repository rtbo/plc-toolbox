use clap::{Parser, Subcommand};
use std::path;

use ua_nsc::opc_bsd;

#[derive(Debug, Subcommand)]
enum Command {
    Typescript {
        #[arg(short, long, value_name = "TYPES_BSD")]
        types_bsd: path::PathBuf,

        #[arg(short, long, value_name = "OUTPUT_DIR")]
        output: Option<path::PathBuf>,
    },
}

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    match run().await {
        Ok(()) => {
            std::process::ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("Error: {:?}", err);
            return std::process::ExitCode::FAILURE;
        }
    }
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::try_parse()?;
    match cli.command {
        Command::Typescript { types_bsd, output } => {
            let types = opc_bsd::parse_types_bsd(&types_bsd).await?;
            let mut output: Box<dyn tokio::io::AsyncWrite + Unpin> = match output {
                Some(path) => Box::new(
                    tokio::fs::OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .create(true)
                        .open(path)
                        .await?,
                ),
                None => Box::new(tokio::io::stdout()),
            };
            ua_nsc::typescript::generate_ts_types(&types, &mut output).await?;
        }
    }
    Ok(())
}
