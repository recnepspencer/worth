//! Conservative record provenance, not exact authored field-path relevance.
use worth_execution::ChargedBytes;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AuthoredSourceRecordUse {
    pub lifecycle: bool,
    pub fields: bool,
}
impl ChargedBytes for AuthoredSourceRecordUse {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
impl AuthoredSourceRecordUse {
    pub const LIFECYCLE: Self = Self {
        lifecycle: true,
        fields: false,
    };
    pub const FIELDS: Self = Self {
        lifecycle: false,
        fields: true,
    };
    pub fn union(self, other: Self) -> Self {
        Self {
            lifecycle: self.lifecycle || other.lifecycle,
            fields: self.fields || other.fields,
        }
    }
}
