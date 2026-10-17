use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Value {
    Unit,
    Text(String),
    Bytes(Vec<u8>),
    Int(i128),
    Bool(bool),
    Optional(Option<Box<Value>>),
    Result(Result<Box<Value>, String>),
    OutputPath(PathBuf),
    Toolchain(PathBuf),
    Command(PathBuf, Vec<String>),
    Output(Vec<u8>),
    Pieces(Vec<String>, usize),
}

impl Value {
    pub(super) fn text(&self) -> Result<String, String> {
        match self {
            Self::Text(value) => Ok(value.clone()),
            Self::Int(value) => Ok(value.to_string()),
            Self::Bool(value) => Ok(value.to_string()),
            _ => Err("generator value is not primitive text".into()),
        }
    }
    pub(super) fn bytes(&self) -> Result<Vec<u8>, String> {
        match self {
            Self::Text(value) => Ok(value.as_bytes().to_vec()),
            Self::Bytes(value) => Ok(value.clone()),
            _ => Err("generator write requires byte/string content".into()),
        }
    }
}

pub(super) enum Fault {
    Error(String),
    Return(Value),
}
impl From<String> for Fault {
    fn from(error: String) -> Self {
        Self::Error(error)
    }
}
impl From<&str> for Fault {
    fn from(error: &str) -> Self {
        Self::Error(error.into())
    }
}
