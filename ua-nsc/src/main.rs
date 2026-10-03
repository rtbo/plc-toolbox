use clap::{Parser, Subcommand};
use std::{collections::HashMap, path};

#[derive(Debug, Subcommand)]
enum Command {
    TsTypes {
        #[arg(short, long, value_name = "TYPES_BSD")]
        types_bsd: path::PathBuf,

        #[arg(short, long, value_name = "NODE_IDS_CSV")]
        node_ids_csv: Option<path::PathBuf>,

        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<path::PathBuf>,
    },
    TsNs {
        #[arg(short, long, value_name = "NODE_IDS_CSV")]
        node_ids_csv: path::PathBuf,

        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<path::PathBuf>,
    },
    /// Generate Typescript files for the NS0 namespace.
    /// This will generate the following files:
    /// - ns0.ts: The Typescript file containing all the NodeIds of the NS0 namespace
    /// - status_codes.ts: The Typescript file containing the status codes
    /// - attribute_ids.ts: The Typescript file containing the attribute ids
    /// - types.ts: The Typescript file containing the NS0 and builtin types
    TsNs0 {
        #[arg(short, long, value_name = "TYPES_BSD")]
        types_bsd: Option<path::PathBuf>,

        #[arg(short, long, value_name = "NODE_IDS_CSV")]
        node_ids_csv: Option<path::PathBuf>,

        #[arg(short, long, value_name = "ATTR_IDS_CSV")]
        attr_ids: Option<path::PathBuf>,

        #[arg(short, long, value_name = "STATUS_CODES_CSV")]
        status_codes: Option<path::PathBuf>,

        #[arg(long, value_name = "SCHEMA_DIR", help = "Path to the directory containing the OPC UA schema files. If not provided, individual file paths must be provided.")]
        schema_dir: Option<path::PathBuf>,

        #[arg(short, long, value_name = "OUTPUT_DIR")]
        output_dir: path::PathBuf,
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
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {:?}", err);
            return std::process::ExitCode::FAILURE;
        }
    }
}

async fn create_new_file(
    path: &path::Path,
) -> anyhow::Result<tokio::io::BufWriter<tokio::fs::File>> {
    let file = tokio::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .await?;
    Ok(tokio::io::BufWriter::new(file))
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::try_parse()?;
    match cli.command {
        Command::TsTypes {
            types_bsd,
            node_ids_csv,
            output,
        } => {
            let types = ua_nsc::parse::opc_bsd::parse_types_bsd(&types_bsd).await?;
            let ua_type_ids = if let Some(csv_path) = node_ids_csv {
                let node_ids = ua_nsc::parse::node_ids::parse_csv(&csv_path).await?;
                node_ids
                    .into_iter()
                    .map(|node_id| (node_id.name.clone(), node_id))
                    .collect::<HashMap<_, _>>()
            } else {
                HashMap::new()
            };
            let mut output: Box<dyn tokio::io::AsyncWrite + Unpin> = match output {
                Some(path) => Box::new(create_new_file(&path).await?),
                None => Box::new(tokio::io::stdout()),
            };
            ua_nsc::ts::types::generate(&types, &ua_type_ids, &mut output).await?;
        }
        Command::TsNs {
            node_ids_csv,
            output,
        } => {
            let node_ids = ua_nsc::parse::node_ids::parse_csv(&node_ids_csv).await?;
            let mut output: Box<dyn tokio::io::AsyncWrite + Unpin> = match output {
                Some(path) => Box::new(create_new_file(&path).await?),
                None => Box::new(tokio::io::stdout()),
            };
            ua_nsc::ts::node_ids::generate(&node_ids, &mut output).await?;
        }
        Command::TsNs0 {
            types_bsd,
            node_ids_csv,
            attr_ids,
            status_codes,
            schema_dir,
            output_dir,
        } => {
            let types_bsd_path = match (types_bsd, &schema_dir) {
                (Some(path), _) => path,
                (None, Some(dir)) => dir.join("Opc.Ua.Types.bsd"),
                (None, None) => {
                    return Err(anyhow::anyhow!(
                        "Either --types-bsd or --schema-dir must be provided"
                    ))
                }
            };
            let node_ids_path = match (node_ids_csv, &schema_dir) {
                (Some(path), _) => path,
                (None, Some(dir)) => dir.join("NodeIds.csv"),
                (None, None) => {
                    return Err(anyhow::anyhow!(
                        "Either --node-ids or --schema-dir must be provided"
                    ))
                }
            };
            let attr_ids_path = match (attr_ids, &schema_dir) {
                (Some(path), _) => path,
                (None, Some(dir)) => dir.join("AttributeIds.csv"),
                (None, None) => {
                    return Err(anyhow::anyhow!(
                        "Either --attr-ids or --schema-dir must be provided"
                    ))
                }
            };
            let status_codes_path = match (status_codes, &schema_dir) {
                (Some(path), _) => path,
                (None, Some(dir)) => dir.join("StatusCode.csv"),
                (None, None) => {
                    return Err(anyhow::anyhow!(
                        "Either --status-codes or --schema-dir must be provided"
                    ))
                }
            };

            println!("Reading {}", types_bsd_path.display());
            let types = ua_nsc::parse::opc_bsd::parse_types_bsd(&types_bsd_path).await?;
            println!("Reading {}", node_ids_path.display());
            let node_ids = ua_nsc::parse::node_ids::parse_csv(&node_ids_path).await?;
            println!("Reading {}", attr_ids_path.display());
            let attr_ids = ua_nsc::parse::attribute_ids::parse_csv(&attr_ids_path).await?;
            println!("Reading {}", status_codes_path.display());
            let status_codes = ua_nsc::parse::status_codes::parse_csv(&status_codes_path).await?;

            println!("Creating {}", output_dir.display());
            tokio::fs::create_dir_all(&output_dir).await?;

            // Generate ns0.ts
            let ns0_path = output_dir.join("ns0.ts");
            println!("Writing {}", ns0_path.display());
            let mut ns0_file = create_new_file(&ns0_path).await?;
            ua_nsc::ts::node_ids::generate(&node_ids, &mut ns0_file).await?;

            // Generate attribute_ids.ts
            let attr_ids_path = output_dir.join("attribute_ids.ts");
            println!("Writing {}", attr_ids_path.display());
            let mut attr_ids_file = create_new_file(&attr_ids_path).await?;
            ua_nsc::ts::attribute_ids::generate(&attr_ids, &mut attr_ids_file).await?;

            // Generate status_codes.ts
            let status_codes_path = output_dir.join("status_codes.ts");
            println!("Writing {}", status_codes_path.display());
            let mut status_codes_file = create_new_file(&status_codes_path).await?;
            ua_nsc::ts::status_codes::generate(&status_codes, &mut status_codes_file).await?;

            // Generate types.ts
            let types_path = output_dir.join("types.ts");
            println!("Writing {}", types_path.display());
            let mut types_file = create_new_file(&types_path).await?;
            let ua_type_ids = node_ids
                .into_iter()
                .map(|node_id| (node_id.name.clone(), node_id))
                .collect::<HashMap<_, _>>();
            ua_nsc::ts::types::generate(&types, &ua_type_ids, &mut types_file).await?;
        }
    }
    Ok(())
}
