use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AspectKey(String);

impl AspectKey {
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if value.trim().is_empty() || value.contains(char::is_whitespace) {
            None
        } else {
            Some(Self(value))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.0.capacity()
    }

    pub(crate) fn visit_allocations(
        &self,
        unique: bool,
        visitor: &mut dyn crate::aspects::FoundationalAllocationVisitor,
    ) {
        crate::aspects::allocation_visitor::visit_string_buffer(&self.0, unique, visitor);
    }
}
