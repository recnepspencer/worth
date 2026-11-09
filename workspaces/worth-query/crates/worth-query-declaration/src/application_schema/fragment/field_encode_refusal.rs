//! A single-field encoder retains its exact path and its binding's typed cause.
use super::super::ApplicationValueEncodeDenial;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldEncodeRefusal {
    path: String,
    cause: ApplicationValueEncodeDenial,
}
impl FieldEncodeRefusal {
    pub(crate) fn new(path: String, cause: ApplicationValueEncodeDenial) -> Self {
        Self { path, cause }
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn cause(&self) -> &ApplicationValueEncodeDenial {
        &self.cause
    }
}
impl std::fmt::Display for FieldEncodeRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "field encoding refused at {}: {:?}",
            self.path, self.cause
        )
    }
}
impl std::error::Error for FieldEncodeRefusal {}
