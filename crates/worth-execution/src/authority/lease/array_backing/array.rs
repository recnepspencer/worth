use super::super::fixed_backing::OwnedFixedBacking;
use super::ExecutionArrayIntoIter;
use std::ops::Deref;

/// Immutable move-only custody of the same allocation admitted by the builder.
/// Sharing is explicit through Arc<ExecutionArray<T>>, without cloning T or
/// issuing another ticket. Retaining custody grants no execution authority.
/// Interior mutability inside T remains T's responsibility.
///
/// ```compile_fail
/// use worth_execution::ExecutionArray;
/// fn duplicate(array: &ExecutionArray<u8>) -> ExecutionArray<u8> { array.clone() }
/// ```
/// ```compile_fail
/// use worth_execution::ExecutionArray;
/// fn rewrite(array: &mut ExecutionArray<u8>) { array[0] = 1; }
/// ```
pub struct ExecutionArray<T> {
    backing: OwnedFixedBacking<T>,
}

impl<T> ExecutionArray<T> {
    pub(super) fn from_owned(backing: OwnedFixedBacking<T>) -> Self {
        Self { backing }
    }
    pub fn elements(&self) -> &[T] {
        self.backing.elements()
    }
    pub fn len(&self) -> usize {
        self.elements().len()
    }
    pub fn is_empty(&self) -> bool {
        self.elements().is_empty()
    }
    /// None for explicit system allocation; Some(0) for leased empty/ZST arrays.
    pub fn charged_payload_bytes(&self) -> Option<u64> {
        self.backing.charged_payload_bytes()
    }
}
impl<T> Deref for ExecutionArray<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.elements()
    }
}
impl<T> AsRef<[T]> for ExecutionArray<T> {
    fn as_ref(&self) -> &[T] {
        self.elements()
    }
}
impl<T: PartialEq> PartialEq for ExecutionArray<T> {
    fn eq(&self, other: &Self) -> bool {
        self.elements() == other.elements()
    }
}
impl<T: Eq> Eq for ExecutionArray<T> {}
impl<T> IntoIterator for ExecutionArray<T> {
    type Item = T;
    type IntoIter = ExecutionArrayIntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        ExecutionArrayIntoIter::from_owned(self.backing.into_iter())
    }
}
impl<'a, T> IntoIterator for &'a ExecutionArray<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.elements().iter()
    }
}
impl<T> std::fmt::Debug for ExecutionArray<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionArray")
            .field("element_count", &self.len())
            .finish_non_exhaustive()
    }
}
