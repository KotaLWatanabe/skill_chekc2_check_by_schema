#[derive(Debug, PartialEq)]
pub enum CheckType {
    Bool, Integer, Str
}
impl CheckType {
    pub fn from_type_str(s: &str) -> Option<CheckType> {
        match s {
            "bool" => Some(CheckType::Bool),
            "integer" => Some(CheckType::Integer),
            "string" => Some(CheckType::Str),
            _ => None
        }
    }
}
