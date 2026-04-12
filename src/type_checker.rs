use crate::check_type::CheckType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeError {
    pub message: String,
}

pub fn check(value: &str, file_type: &CheckType) -> Result<(), TypeError> {
    match file_type {
        CheckType::Bool => {
            match value {
                "true" | "false" => Ok(()),
                _ => Err(TypeError { message: format!("Expected a boolean value, got '{}'", value) }),
            }
        },
        CheckType::Integer => {
            match value.parse::<i64>() {
                Ok(_) => Ok(()),
                Err(_) => Err(TypeError { message: format!("Expected an integer value, got '{}'", value) }),
            }
        },
        CheckType::Str => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bool型_true_falseを許可する() {
        assert_eq!(check("true", &CheckType::Bool), Ok(()));
        assert_eq!(check("false", &CheckType::Bool), Ok(()));
    }

    #[test]
    fn bool型_不正値でエラー() {
        let result = check("yes", &CheckType::Bool);

        assert!(result.is_err());
        assert_eq!(
            result.err().map(|e| e.message),
            Some("Expected a boolean value, got 'yes'".to_string())
        );
    }

    #[test]
    fn integer型_整数を許可する() {
        assert_eq!(check("11111111111", &CheckType::Integer), Ok(()));
        assert_eq!(check("0", &CheckType::Integer), Ok(()));
        assert_eq!(check("-42", &CheckType::Integer), Ok(()));
    }

    #[test]
    fn integer型_不正値でエラー() {
        let result = check("12.5", &CheckType::Integer);

        assert!(result.is_err());
        assert_eq!(
            result.err().map(|e| e.message),
            Some("Expected an integer value, got '12.5'".to_string())
        );
    }

    #[test]
    fn string型は任意文字列を許可する() {
        assert_eq!(check("#@$%!*^())[]\'", &CheckType::Str), Ok(()));
        assert_eq!(check("", &CheckType::Str), Ok(()));
    }
}
