//! Relation kind and endpoint roles resolved from the installed operation reads.
use worth_query_declaration::facade::application_aftermath::PortablePreImageRelationLocus;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledPreImageRelationLocus {
    relation: String,
    from: String,
    to: String,
}
impl InstalledPreImageRelationLocus {
    pub(super) fn from_portable(value: &PortablePreImageRelationLocus) -> Self {
        Self {
            relation: value.relation().to_owned(),
            from: value.from().to_owned(),
            to: value.to().to_owned(),
        }
    }
    pub fn relation(&self) -> &str {
        &self.relation
    }
    pub fn from(&self) -> &str {
        &self.from
    }
    pub fn to(&self) -> &str {
        &self.to
    }
}
