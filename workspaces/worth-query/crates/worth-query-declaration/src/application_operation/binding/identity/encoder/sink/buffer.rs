//! Canonical map-entry bytes borrow their parent's controls and allocation posture.
use super::{reserve, CanonicalEncodeError, CanonicalEncodingCharge, Sink};
use std::fmt::Debug;

pub struct EntryBuffer<'a, E: Debug> {
    bytes: Vec<u8>,
    nested_buffered: usize,
    parent: &'a mut dyn Sink<AdmissionError = E>,
}
impl<'a, E: Debug> EntryBuffer<'a, E> {
    pub fn new(parent: &'a mut dyn Sink<AdmissionError = E>) -> Self {
        Self {
            bytes: Vec::new(),
            nested_buffered: 0,
            parent,
        }
    }
    pub fn finish(self) -> (Vec<u8>, usize) {
        (self.bytes, self.nested_buffered)
    }
}
impl<E: Debug> Sink for EntryBuffer<'_, E> {
    type AdmissionError = E;
    fn tracks_work(&self) -> bool {
        self.parent.tracks_work()
    }
    fn poll(&mut self) -> Result<(), CanonicalEncodeError<E>> {
        self.parent.poll()
    }
    fn ensure_active(&self) -> Result<(), CanonicalEncodeError<E>> {
        self.parent.ensure_active()
    }
    fn fail_capacity(&mut self) -> CanonicalEncodeError<E> {
        self.parent.fail_capacity()
    }
    fn fail_allocation(&mut self) -> CanonicalEncodeError<E> {
        self.parent.fail_allocation()
    }
    fn admit(&mut self, charge: CanonicalEncodingCharge) -> Result<(), CanonicalEncodeError<E>> {
        self.parent.admit(charge)
    }
    fn put(&mut self, bytes: &[u8]) -> Result<(), CanonicalEncodeError<E>> {
        reserve(self.parent, &mut self.bytes, bytes.len())?;
        if self.tracks_work() {
            let copied = u64::try_from(bytes.len()).map_err(|_| self.parent.fail_capacity())?;
            self.parent.admit(CanonicalEncodingCharge::Work(copied))?;
        } else {
            self.poll()?;
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    fn buffered(&mut self, bytes: usize) -> Result<(), CanonicalEncodeError<E>> {
        self.parent.ensure_active()?;
        if self.tracks_work() {
            self.nested_buffered = self
                .nested_buffered
                .checked_add(bytes)
                .ok_or_else(|| self.parent.fail_capacity())?;
        }
        Ok(())
    }
}
