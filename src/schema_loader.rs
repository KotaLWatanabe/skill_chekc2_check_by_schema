use std::fs;
use std::path::Path;
use crate::schema_parser::{parse_schema, SchemaMap, SchemaParseError};

#[derive(Debug)]
pub enum SchemaLoadError {
    Io(std::io::Error),
    Parse(SchemaParseError),
}

impl From<std::io::Error> for SchemaLoadError {
    fn from(value: std::io::Error) -> Self {
        SchemaLoadError::Io(value)
    }
}

impl From<SchemaParseError> for SchemaLoadError {
    fn from(value: SchemaParseError) -> Self {
        SchemaLoadError::Parse(value)
    }
}

impl std::fmt::Display for SchemaLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaLoadError::Io(err) => write!(f, "I/O error: {}", err),
            SchemaLoadError::Parse(err) => write!(
                f,
                "schema parse error at line {}: {} (content: {})",
                err.line, err.message, err.content
            ),
        }
    }
}

impl std::error::Error for SchemaLoadError {}

#[inline]
fn load_schema_file(path: impl AsRef<Path>) -> std::io::Result<String> {
    fs::read_to_string(path)
}

pub fn load_and_parse_schema(path: impl AsRef<Path>) -> Result<SchemaMap, SchemaLoadError> {
    let content = load_schema_file(path)?;
    let parsed = parse_schema(&content)?;
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use skill_chekc1_conf_load::parse_conf_file;
    use crate::check_type::CheckType;
    use crate::schema_parser::check_parsed_map_against_schema;

    #[test]
    fn sample_schemaを読み込んでcheck_typeへ変換できる() {
        let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data")
            .join("schma")
            .join("sample.schema");

        let schema = load_and_parse_schema(schema_path).expect("schema should load");

        assert_eq!(schema.get("endpoint"), Some(&CheckType::Str));
        assert_eq!(schema.get("debug"), Some(&CheckType::Bool));
        assert_eq!(schema.get("log.file"), Some(&CheckType::Str));
        assert_eq!(schema.get("retry"), Some(&CheckType::Integer));
    }

    #[test]
    fn sample_schemaをtype_checkerに渡して検証できる() {
        let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data")
            .join("schma")
            .join("sample.schema");
        let conf_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data")
            .join("conf")
            .join("sample3.conf");

        let schema = load_and_parse_schema(schema_path).expect("schema should load");
        let parsed = parse_conf_file(conf_path).expect("conf should load");

        let result = check_parsed_map_against_schema(&parsed, &schema);
        assert!(result.is_ok());
    }
}
