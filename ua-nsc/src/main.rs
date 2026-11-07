use std::collections::HashMap;
use std::path;

use clap::{Parser, Subcommand};

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
    /// - status_code.ts: The Typescript file containing the status codes
    /// - attribute_ids.ts: The Typescript file containing the attribute ids
    /// - types.ts: The Typescript file containing the NS0 and builtin types
    TsNs0 {
        #[arg(short, long, value_name = "TYPES_BSD")]
        types_bsd: Option<path::PathBuf>,

        #[arg(short, long, value_name = "NODE_IDS_CSV")]
        node_ids_csv: Option<path::PathBuf>,

        #[arg(short, long, value_name = "ATTR_IDS_CSV")]
        attr_ids: Option<path::PathBuf>,

        #[arg(short, long, value_name = "STATUS_CODE_CSV")]
        status_code: Option<path::PathBuf>,

        #[arg(
            long,
            value_name = "SCHEMA_DIR",
            help = "Path to the directory containing the OPC UA schema files. If not provided, individual file paths must be provided."
        )]
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

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {:?}", err);
            return std::process::ExitCode::FAILURE;
        }
    }
}

fn create_new_file(path: &path::Path) -> anyhow::Result<std::io::BufWriter<std::fs::File>> {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;
    Ok(std::io::BufWriter::new(file))
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::try_parse()?;
    match cli.command {
        Command::TsTypes {
            types_bsd,
            node_ids_csv,
            output,
        } => {
            let types = ua_nsc::parse::types::parse_types_bsd(&types_bsd)?;
            let ua_type_ids = if let Some(csv_path) = node_ids_csv {
                let node_ids = ua_nsc::parse::node_ids::parse_csv(&csv_path)?;
                node_ids
                    .into_iter()
                    .map(|node_id| (node_id.name.clone(), node_id))
                    .collect::<HashMap<_, _>>()
            } else {
                HashMap::new()
            };
            let mut output: Box<dyn std::io::Write + Unpin> = match output {
                Some(path) => Box::new(create_new_file(&path)?),
                None => Box::new(std::io::stdout()),
            };
            ua_nsc::ts::types::generate(&types, &ua_type_ids, &mut output)?;
        }
        Command::TsNs {
            node_ids_csv,
            output,
        } => {
            let node_ids = ua_nsc::parse::node_ids::parse_csv(&node_ids_csv)?;
            let mut output: Box<dyn std::io::Write + Unpin> = match output {
                Some(path) => Box::new(create_new_file(&path)?),
                None => Box::new(std::io::stdout()),
            };
            ua_nsc::ts::node_ids::generate(&node_ids, &mut output)?;
        }
        Command::TsNs0 {
            types_bsd,
            node_ids_csv,
            attr_ids,
            status_code,
            schema_dir,
            output_dir,
        } => {
            let types_bsd_path = match (types_bsd, &schema_dir) {
                (Some(path), _) => path,
                (None, Some(dir)) => dir.join("Opc.Ua.Types.bsd"),
                (None, None) => {
                    return Err(anyhow::anyhow!(
                        "Either --types-bsd or --schema-dir must be provided"
                    ));
                }
            };
            let node_ids_path = match (node_ids_csv, &schema_dir) {
                (Some(path), _) => path,
                (None, Some(dir)) => dir.join("NodeIds.csv"),
                (None, None) => {
                    return Err(anyhow::anyhow!(
                        "Either --node-ids or --schema-dir must be provided"
                    ));
                }
            };
            let attr_ids_path = match (attr_ids, &schema_dir) {
                (Some(path), _) => path,
                (None, Some(dir)) => dir.join("AttributeIds.csv"),
                (None, None) => {
                    return Err(anyhow::anyhow!(
                        "Either --attr-ids or --schema-dir must be provided"
                    ));
                }
            };
            let status_code_path = match (status_code, &schema_dir) {
                (Some(path), _) => path,
                (None, Some(dir)) => dir.join("StatusCode.csv"),
                (None, None) => {
                    return Err(anyhow::anyhow!(
                        "Either --status-codes or --schema-dir must be provided"
                    ));
                }
            };

            println!("Reading {}", types_bsd_path.display());
            let types = ua_nsc::parse::types::parse_types_bsd(&types_bsd_path)?;
            println!("Reading {}", node_ids_path.display());
            let node_ids = ua_nsc::parse::node_ids::parse_csv(&node_ids_path)?;
            println!("Reading {}", attr_ids_path.display());
            let attr_ids = ua_nsc::parse::attribute_ids::parse_csv(&attr_ids_path)?;
            println!("Reading {}", status_code_path.display());
            let status_code = ua_nsc::parse::status_code::parse_csv(&status_code_path)?;

            println!("Creating {}", output_dir.display());
            std::fs::create_dir_all(&output_dir)?;

            // Generate ns0.ts
            let ns0_path = output_dir.join("ns0.ts");
            println!("Writing {}", ns0_path.display());
            let mut ns0_file = create_new_file(&ns0_path)?;
            ua_nsc::ts::node_ids::generate(&node_ids, &mut ns0_file)?;

            // Generate attribute_ids.ts
            let attr_ids_path = output_dir.join("attribute_ids.ts");
            println!("Writing {}", attr_ids_path.display());
            let mut attr_ids_file = create_new_file(&attr_ids_path)?;
            ua_nsc::ts::attribute_ids::generate(&attr_ids, &mut attr_ids_file)?;

            // Generate status_code.ts
            let status_code_path = output_dir.join("status_code.ts");
            println!("Writing {}", status_code_path.display());
            let mut status_code_file = create_new_file(&status_code_path)?;
            ua_nsc::ts::status_code::generate(&status_code, &mut status_code_file)?;

            // Generate types.ts
            let types_path = output_dir.join("types.ts");
            println!("Writing {}", types_path.display());
            let mut types_file = create_new_file(&types_path)?;
            let ua_type_ids = node_ids
                .into_iter()
                .map(|node_id| (node_id.name.clone(), node_id))
                .collect::<HashMap<_, _>>();
            ua_nsc::ts::types::generate(&types, &ua_type_ids, &mut types_file)?;
        }
    }
    Ok(())
}
