#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalizationRuleVersion(String);

impl CanonicalizationRuleVersion {
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.0.capacity()
    }

    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if value.is_empty() || value.trim() != value || value.chars().any(char::is_whitespace) {
            None
        } else {
            Some(Self(value))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
