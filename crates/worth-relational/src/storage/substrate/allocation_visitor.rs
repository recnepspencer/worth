//! Physical owner traversal. Visitors may prune already-seen or externally retained owners.
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StorageAllocationObservation {
    pub(crate) id: u64,
    pub(crate) bytes: u64,
    pub(crate) unique: bool,
}

pub(crate) trait StorageAllocationVisitor {
    /// Return true to visit allocations owned by this allocation as well.
    fn visit(&mut self, allocation: StorageAllocationObservation) -> bool;
}

pub(crate) fn visit_inline_value<T>(
    _value: &T,
    allocation: StorageAllocationObservation,
    visitor: &mut dyn StorageAllocationVisitor,
) {
    visitor.visit(allocation);
}

pub(crate) fn visit_new_inline_value<T>(
    value: &T,
    _previous: Option<&T>,
    allocation: StorageAllocationObservation,
    visitor: &mut dyn StorageAllocationVisitor,
) {
    visit_inline_value(value, allocation, visitor);
}

impl<T> super::StorageAllocation<T> {
    pub(crate) fn observation(&self, ancestor_unique: bool) -> StorageAllocationObservation {
        StorageAllocationObservation {
            id: self.id(),
            bytes: Self::layout_bytes(),
            unique: ancestor_unique && self.is_unique(),
        }
    }
}
