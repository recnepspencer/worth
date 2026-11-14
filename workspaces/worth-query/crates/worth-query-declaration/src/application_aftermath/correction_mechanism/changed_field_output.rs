//! Descriptive identity of the one preserved output whose changed fields are retained.

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PortableChangedFieldOutput {
    role: String,
    entity: String,
}

impl PortableChangedFieldOutput {
    /// Untrusted contract data; installation checks the actual output declaration.
    pub fn from_untrusted_fields(role: String, entity: String) -> Self {
        Self { role, entity }
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub fn entity(&self) -> &str {
        &self.entity
    }
}
