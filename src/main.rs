mod check_type;
mod schema_loader;
mod schema_parser;
mod type_checker;

use crate::schema_loader::load_and_parse_schema;
use crate::schema_parser::check_parsed_map_against_schema;
use clap::Parser;
use skill_chekc1_conf_load::parse_conf_file;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "check_conf")]
#[command(about = "Schema validator for sysctl config files", long_about = None)]
struct Args {
    #[arg(value_name = "Conf File Path")]
    conf_path: PathBuf,
    #[arg(value_name = "Schema File Path")]
    schema_path: PathBuf,
}

#[derive(Debug)]
enum AppError {
    InputPath(InputPathError),
    ParseConf(skill_chekc1_conf_load::ParseFileError),
    ParseSchema(crate::schema_loader::SchemaLoadError),
    Validation(Vec<crate::type_checker::TypeError>),
}

#[derive(Debug)]
struct InputPathError {
    path: PathBuf,
    kind: InputPathErrorKind,
}

#[derive(Debug)]
enum InputPathErrorKind {
    NotFound,
    NotAFile,
    Io(io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InputPath(err) => write!(f, "{}", err),
            AppError::ParseConf(err) => write!(f, "Error parsing configuration file: {}", err),
            AppError::ParseSchema(err) => write!(f, "Error at parsing schema file: {}", err),
            AppError::Validation(errors) => {
                writeln!(f, "Validation errors:")?;
                for error in errors {
                    writeln!(f, "- {}", error.message)?;
                }
                Ok(())
            }
        }
    }
}

impl fmt::Display for InputPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            InputPathErrorKind::NotFound => {
                write!(f, "Error: File not found: {}", self.path.display())
            }
            InputPathErrorKind::NotAFile => {
                write!(f, "Error: Path is not a file: {}", self.path.display())
            }
            InputPathErrorKind::Io(err) => {
                write!(
                    f,
                    "Error: Failed to access path {}: {}",
                    self.path.display(),
                    err
                )
            }
        }
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), AppError> {
    let args = Args::parse();

    validate_input_file(&args.conf_path).map_err(AppError::InputPath)?;
    validate_input_file(&args.schema_path).map_err(AppError::InputPath)?;

    let parsed = parse_conf_file(&args.conf_path).map_err(AppError::ParseConf)?;
    let schema = load_and_parse_schema(&args.schema_path).map_err(AppError::ParseSchema)?;

    check_parsed_map_against_schema(&parsed, &schema).map_err(AppError::Validation)?;

    Ok(())
}

fn validate_input_file(path: &Path) -> Result<(), InputPathError> {
    match path.metadata() {
        Ok(metadata) if metadata.is_file() => Ok(()),
        Ok(_) => Err(InputPathError {
            path: path.to_path_buf(),
            kind: InputPathErrorKind::NotAFile,
        }),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Err(InputPathError {
            path: path.to_path_buf(),
            kind: InputPathErrorKind::NotFound,
        }),
        Err(err) => Err(InputPathError {
            path: path.to_path_buf(),
            kind: InputPathErrorKind::Io(err),
        }),
    }
}
