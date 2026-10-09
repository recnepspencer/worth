use super::super::ExecutionMemoryReservation;
use allocator_api2::vec::{IntoIter, Vec};
use std::iter::FusedIterator;

/// Field drop order keeps the linear charge until elements and storage are gone.
pub(in super::super) struct OwnedFixedBacking<T> {
    elements: Vec<T>,
    reservation: Option<ExecutionMemoryReservation>,
}

impl<T> OwnedFixedBacking<T> {
    pub(super) fn new(elements: Vec<T>, reservation: Option<ExecutionMemoryReservation>) -> Self {
        Self {
            elements,
            reservation,
        }
    }
    pub(super) fn push(&mut self, value: T) {
        self.elements.push(value);
    }
    pub(in super::super) fn elements(&self) -> &[T] {
        &self.elements
    }
    pub(super) fn capacity(&self) -> usize {
        self.elements.capacity()
    }
    pub(in super::super) fn charged_payload_bytes(&self) -> Option<u64> {
        self.reservation
            .as_ref()
            .map(ExecutionMemoryReservation::bytes)
    }
    pub(in super::super) fn into_iter(self) -> OwnedFixedIntoIter<T> {
        let Self {
            elements,
            reservation,
        } = self;
        let elements = if std::mem::size_of::<T>() == 0 {
            IteratingElements::ZeroSized(elements)
        } else {
            IteratingElements::Sized(elements.into_iter())
        };
        OwnedFixedIntoIter {
            elements,
            reservation,
        }
    }
}

impl OwnedFixedBacking<u8> {
    pub(super) fn append_bytes(&mut self, bytes: &[u8]) {
        self.elements.extend_from_slice(bytes);
    }
    pub(super) fn written_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.elements
    }
}

/// Remaining elements and storage drop before the retained reservation.
/// Exhausting iteration does not free the backing; dropping the iterator does.
pub(in super::super) struct OwnedFixedIntoIter<T> {
    elements: IteratingElements<T>,
    reservation: Option<ExecutionMemoryReservation>,
}

enum IteratingElements<T> {
    Sized(IntoIter<T>),
    // Keep the aligned Vec pointer: pinned 0.2.21 IntoIter increments the ZST
    // pointer by a byte, then uses it for slices/drop. Safe pop avoids that
    // mechanism. ZST values have no distinct stored representation, so pop
    // serves either direction while preserving the exact logical count.
    ZeroSized(Vec<T>),
}

impl<T> OwnedFixedIntoIter<T> {
    pub(in super::super) fn elements(&self) -> &[T] {
        match &self.elements {
            IteratingElements::Sized(elements) => elements.as_slice(),
            IteratingElements::ZeroSized(elements) => elements,
        }
    }
    pub(in super::super) fn charged_payload_bytes(&self) -> Option<u64> {
        self.reservation
            .as_ref()
            .map(ExecutionMemoryReservation::bytes)
    }
}
impl<T> Iterator for OwnedFixedIntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        match &mut self.elements {
            IteratingElements::Sized(elements) => elements.next(),
            IteratingElements::ZeroSized(elements) => elements.pop(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = match &self.elements {
            IteratingElements::Sized(elements) => elements.len(),
            IteratingElements::ZeroSized(elements) => elements.len(),
        };
        (remaining, Some(remaining))
    }
}
impl<T> DoubleEndedIterator for OwnedFixedIntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        match &mut self.elements {
            IteratingElements::Sized(elements) => elements.next_back(),
            IteratingElements::ZeroSized(elements) => elements.pop(),
        }
    }
}
impl<T> ExactSizeIterator for OwnedFixedIntoIter<T> {}
impl<T> FusedIterator for OwnedFixedIntoIter<T> {}
