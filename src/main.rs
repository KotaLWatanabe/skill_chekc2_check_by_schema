mod check_type;
mod schema_loader;
mod schema_parser;
mod type_checker;

use crate::schema_loader::load_and_parse_schema;
use crate::schema_parser::check_parsed_map_against_schema;
use clap::Parser;
use skill_chekc1_conf_load::parse_conf_file;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "check_conf")]
#[command(about = "Schema validator for sysctl config files", long_about = None)]
struct Args {
    #[arg(value_name = "Conf File Path")]
    conf_path: PathBuf,
    #[arg(value_name = "Schema File Path")]
    schema_path: PathBuf,
}

fn main() {
    let args = Args::parse();

    if !args.conf_path.exists() {
        eprintln!("Error: File not found: {}", args.conf_path.display());
        std::process::exit(1);
    }
    if !args.schema_path.exists() {
        eprintln!("Error: File not found: {}", args.schema_path.display());
        std::process::exit(1);
    }

    if !args.conf_path.is_file() {
        eprintln!("Error: Path is not a file: {}", args.conf_path.display());
        std::process::exit(1);
    }
    if !args.schema_path.is_file() {
        eprintln!("Error: Path is not a file: {}", args.schema_path.display());
        std::process::exit(1);
    }

    let parsed = parse_conf_file(args.conf_path).unwrap_or_else(|err| {
        eprintln!("Error parsing configuration file: {}", err);
        std::process::exit(1);
    });
    let schema = load_and_parse_schema(args.schema_path).unwrap_or_else(|err| {
        eprintln!("Error at parsing schema file: {}", err);
        std::process::exit(1);
    });

    check_parsed_map_against_schema(&parsed, &schema).unwrap_or_else(|errors| {
        eprintln!("Validation errors:");
        for error in errors {
            eprintln!("- {}", error.message);
        }
        std::process::exit(1);
    });
}
