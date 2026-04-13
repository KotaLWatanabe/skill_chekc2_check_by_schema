use crate::check_type::CheckType;
use crate::type_checker::{TypeError, check};
use skill_chekc1_conf_load::ParsedMap;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::panic;

const SCHEMA_SEPARATOR: char = ':';

#[derive(Debug)]
pub struct SchemaParseError {
    pub line: usize,
    pub message: String,
    pub content: String,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SchemaPath(String);

impl SchemaPath {
    pub fn new(path: &str) -> Self {
        assert!(
            !path.contains(SCHEMA_SEPARATOR),
            "SchemaPath must not contain SCHEMA_SEPARATOR"
        );
        Self(path.to_string())
    }
}

impl Borrow<str> for SchemaPath {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SchemaPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type SchemaMap = HashMap<SchemaPath, CheckType>;

fn is_ignored_line(line: &str) -> bool {
    line.is_empty() || line.starts_with('#') || line.starts_with(';')
}

pub fn parse_schema(input: &str) -> Result<SchemaMap, SchemaParseError> {
    let mut schema_map = SchemaMap::new();

    for (idx, raw_line) in input.lines().enumerate() {
        let trimmed = raw_line.trim();
        if is_ignored_line(trimmed) {
            continue;
        }

        let (key_raw, type_raw) = match trimmed.split_once(SCHEMA_SEPARATOR) {
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

        schema_map.insert(SchemaPath::new(key), check_type);
    }

    Ok(schema_map)
}

pub fn check_parsed_map_against_schema(
    parsed: &ParsedMap,
    schema: &SchemaMap,
) -> Result<(), Vec<TypeError>> {
    let mut errors = Vec::new();

    for (path, check_type) in schema {
        let get_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            parsed.get_by_path(path.borrow())
        }));
        let value = match get_result {
            Err(_) => {
                errors.push(TypeError {
                    message: format!("Panic occurred while accessing path `{}`", path),
                });
                continue;
            }
            Ok(Some(v)) => v,
            Ok(None) => {
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
            (Some(..), Some(true)) => {
                continue;
            }
            _ => {
                errors.push(TypeError {
                    message: format!("Value not found at path `{}` in parsed config", path),
                });
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
pub(crate) mod proptest_generators {
    use super::SchemaPath;
    use proptest::prelude::*;
    use proptest::string::string_regex;

    pub(crate) fn gen_schema_path() -> impl Strategy<Value = SchemaPath> {
        proptest::collection::vec(
            string_regex("[a-zA-Z_][a-zA-Z0-9_]{0,7}").unwrap(),
            1..4,
        )
        .prop_map(|segments| SchemaPath::new(&segments.join(".")))
    }

    pub(crate) fn gen_check_type() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            Just("bool"),
            Just("integer"),
            Just("string"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::proptest_generators::{gen_check_type, gen_schema_path};
    use super::*;
    use crate::check_type::CheckType;
    use proptest::prelude::*;
    use proptest::string::string_regex;
    use std::borrow::Borrow;

    proptest! {
        #[test]
        fn スキーマをパースできる(
            path in gen_schema_path(),
            type_str in gen_check_type(),
        ) {
            let input = format!("{} : {}", path, type_str);
            let result = parse_schema(&input).unwrap();
            let expected = CheckType::from_type_str(type_str).unwrap();
            prop_assert_eq!(result.get::<str>(path.borrow()), Some(&expected));
        }

        #[test]
        fn コメント行とスペースを無視できる(
            path in gen_schema_path(),
            comment in "[^\n]{0,20}",
        ) {
            let input = format!("# {}\n{} : string\n\n; {}", comment, path, comment);
            let result = parse_schema(&input).unwrap();
            prop_assert_eq!(result.len(), 1);
            prop_assert_eq!(result.get::<str>(path.borrow()), Some(&CheckType::Str));
        }
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

    proptest! {
        #[test]
        fn スキーマに対して検証できる(
            path in gen_schema_path(),
            value in string_regex("[!-~]{1,16}").unwrap(),
        ) {
            let schema = {
                let mut s = SchemaMap::new();
                s.insert(SchemaPath::new(path.borrow()), CheckType::Str);
                s
            };
            let parsed = ParsedMap::path_to_value(path.borrow(), &value, false);
            let result = check_parsed_map_against_schema(&parsed, &schema);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn ignore_failureがtrueなら型エラーを数えない(
            path in gen_schema_path(),
        ) {
            let schema = {
                let mut s = SchemaMap::new();
                s.insert(SchemaPath::new(path.borrow()), CheckType::Integer);
                s
            };
            let parsed = ParsedMap::path_to_value(path.borrow(), "not_a_number", true);
            let result = check_parsed_map_against_schema(&parsed, &schema);
            prop_assert!(result.is_ok());
        }
    }

    #[test]
    fn パスが見つからない場合エラー() {
        let schema = {
            let mut s = SchemaMap::new();
            s.insert(SchemaPath::new("endpoint"), CheckType::Str);
            s.insert(SchemaPath::new("missing"), CheckType::Str);
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
            s.insert(SchemaPath::new("retry"), CheckType::Integer);
            s
        };

        let parsed = ParsedMap::path_to_value("retry", "not_a_number", false);

        let result = check_parsed_map_against_schema(&parsed, &schema);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }
}
