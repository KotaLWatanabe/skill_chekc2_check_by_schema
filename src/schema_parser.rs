use crate::check_type::CheckType;
use crate::type_checker::{TypeError, check};
use skill_chekc1_conf_load::ParsedMap;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaParseError {
    pub line: usize,
    pub message: String,
    pub content: String,
}

type SchemePath = String;
pub type SchemaMap = HashMap<SchemePath, CheckType>;

pub fn parse_schema(input: &str) -> Result<SchemaMap, SchemaParseError> {
    let mut schema = SchemaMap::new();

    for (idx, raw_line) in input.lines().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }

        let (key_raw, type_raw) = match trimmed.split_once(':') {
            Some(parts) => parts,
            None => {
                return Err(SchemaParseError {
                    line: idx + 1,
                    message: "invalid schema line; expected `key : type`".to_string(),
                    content: raw_line.to_string(),
                });
            }
        };

        let key = key_raw.trim();
        let type_name = type_raw.trim();

        if key.is_empty() || type_name.is_empty() {
            return Err(SchemaParseError {
                line: idx + 1,
                message: "schema key/type must not be empty".to_string(),
                content: raw_line.to_string(),
            });
        }

        let check_type = CheckType::from_type_str(type_name).ok_or_else(|| SchemaParseError {
            line: idx + 1,
            message: format!("unsupported type `{}`", type_name),
            content: raw_line.to_string(),
        })?;

        schema.insert(key.to_string(), check_type);
    }

    Ok(schema)
}

pub fn check_parsed_map_against_schema(
    parsed: &ParsedMap,
    schema: &SchemaMap,
) -> Result<(), Vec<TypeError>> {
    let mut errors = Vec::new();

    for (path, check_type) in schema {
        let value = match parsed.get_by_path(path) {
            Some(v) => v,
            None => {
                errors.push(TypeError {
                    message: format!("Path `{}` not found in parsed config", path),
                });
                continue;
            }
        };

        match (value.value_as_str(), value.ignore_failure()) {
            (Some(value), Some(false)) => {
                if let Err(err) = check(value, check_type) {
                    errors.push(TypeError {
                        message: format!("{} at `{}`", err.message, path),
                    });
                }
            }
            _ => {
                continue;
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check_type::CheckType;

    #[test]
    fn スキーマをパースできる() {
        let input = "endpoint : string\ndebug : bool\nretry : integer";
        let result = parse_schema(input).unwrap();
        
        assert_eq!(result.get("endpoint"), Some(&CheckType::Str));
        assert_eq!(result.get("debug"), Some(&CheckType::Bool));
        assert_eq!(result.get("retry"), Some(&CheckType::Integer));
    }

    #[test]
    fn コメント行とスペースを無視できる() {
        let input = "# This is a comment\nendpoint : string\n\n; Another comment\ndebug : bool";
        let result = parse_schema(input).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result.get("endpoint"), Some(&CheckType::Str));
        assert_eq!(result.get("debug"), Some(&CheckType::Bool));
    }

    #[test]
    fn コロンがない行でエラー() {
        let input = "endpoint string";
        let result = parse_schema(input);
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.line, 1);
        assert!(err.message.contains("invalid schema line"));
    }

    #[test]
    fn 空のキーでエラー() {
        let input = " : string";
        let result = parse_schema(input);
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("must not be empty"));
    }

    #[test]
    fn 空のタイプでエラー() {
        let input = "endpoint : ";
        let result = parse_schema(input);
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("must not be empty"));
    }

    #[test]
    fn 未サポートのタイプでエラー() {
        let input = "endpoint : unknown";
        let result = parse_schema(input);
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("unsupported type"));
    }

    #[test]
    fn スキーマに対して検証できる() {
        let schema = {
            let mut s = SchemaMap::new();
            s.insert("endpoint".to_string(), CheckType::Str);
            s.insert("debug".to_string(), CheckType::Bool);
            s
        };

        let mut parsed = ParsedMap::new();
        parsed.merge(ParsedMap::path_to_value("endpoint", "localhost:3000", false));
        parsed.merge(ParsedMap::path_to_value("debug", "true", false));

        let result = check_parsed_map_against_schema(&parsed, &schema);
        assert!(result.is_ok());
    }

    #[test]
    fn パスが見つからない場合エラー() {
        let schema = {
            let mut s = SchemaMap::new();
            s.insert("endpoint".to_string(), CheckType::Str);
            s.insert("missing".to_string(), CheckType::Str);
            s
        };

        let parsed = ParsedMap::path_to_value("endpoint", "localhost:3000", false);

        let result = check_parsed_map_against_schema(&parsed, &schema);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("not found"));
    }

    #[test]
    fn 型の検証に失敗する() {
        let schema = {
            let mut s = SchemaMap::new();
            s.insert("retry".to_string(), CheckType::Integer);
            s
        };

        let parsed = ParsedMap::path_to_value("retry", "not_a_number", false);

        let result = check_parsed_map_against_schema(&parsed, &schema);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn ignore_failureがtrueなら型エラーを数えない() {
        let schema = {
            let mut s = SchemaMap::new();
            s.insert("retry".to_string(), CheckType::Integer);
            s
        };

        let parsed = ParsedMap::path_to_value("retry", "not_a_number", true);

        let result = check_parsed_map_against_schema(&parsed, &schema);
        assert!(result.is_ok());
    }
}
