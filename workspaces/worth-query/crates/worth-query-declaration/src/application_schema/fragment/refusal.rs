//! Refusals retain the exact flattened field path and the owner's typed cause.

use super::super::{ApplicationValueDecodeDenial, ApplicationValueEncodeDenial};

#[derive(Debug)]
pub struct RecordRefusal {
    path: String,
    kind: RecordRefusalKind,
}

#[derive(Debug)]
pub enum RecordRefusalKind {
    UnknownCase,
    Encode(ApplicationValueEncodeDenial),
    Decode(ApplicationValueDecodeDenial),
    Constructor(Box<dyn std::error::Error + Send + Sync>),
    MissingField,
    DuplicateField,
    UnexpectedField,
    InvalidFieldName,
    UniqueNeedsScalar,
    QueryabilityNeedsScalar,
    WrongValueType,
}

impl RecordRefusal {
    pub(crate) fn new(path: &str, kind: RecordRefusalKind) -> Self {
        Self {
            path: path.to_owned(),
            kind,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn kind(&self) -> &RecordRefusalKind {
        &self.kind
    }
}

impl std::fmt::Display for RecordRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "record refused at {}: {:?}", self.path, self.kind)
    }
}

impl std::error::Error for RecordRefusal {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            RecordRefusalKind::Constructor(cause) => Some(cause.as_ref()),
            _ => None,
        }
    }
}
