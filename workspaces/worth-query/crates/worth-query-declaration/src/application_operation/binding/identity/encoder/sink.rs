//! Fallible canonical byte sinks and their single cumulative admission.

use std::{convert::Infallible, fmt::Debug, mem::size_of};

use serde::ser;
use sha2::{Digest, Sha256};

mod admission;
mod buffer;
pub use admission::{CallbackAdmission, EncodingAdmission, NoAdmission, PollingAdmission};
pub use buffer::EntryBuffer;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalEncodingCharge {
    Work(u64),
    Scratch(u64),
}

#[derive(Debug)]
pub enum CanonicalEncodeError<E: Debug = Infallible> {
    Serialization,
    Admission(E),
    /// The first admission error is retained by the outer owner.
    AdmissionDeferred,
    CapacityOverflow,
    Allocation,
}

impl<E: Debug> std::fmt::Display for CanonicalEncodeError<E> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialization => formatter.write_str("value serialization rejected"),
            Self::Admission(_) => formatter.write_str("canonical encoding admission rejected"),
            Self::AdmissionDeferred => formatter.write_str("canonical encoding admission rejected"),
            Self::CapacityOverflow => formatter.write_str("canonical encoding capacity overflow"),
            Self::Allocation => formatter.write_str("canonical encoding allocation rejected"),
        }
    }
}

impl<E: Debug> std::error::Error for CanonicalEncodeError<E> {}

impl<E: Debug> ser::Error for CanonicalEncodeError<E> {
    fn custom<T: std::fmt::Display>(_: T) -> Self {
        Self::Serialization
    }
}

pub trait Sink {
    type AdmissionError: Debug;

    fn tracks_work(&self) -> bool {
        true
    }
    fn poll(&mut self) -> Result<(), CanonicalEncodeError<Self::AdmissionError>>;
    fn ensure_active(&self) -> Result<(), CanonicalEncodeError<Self::AdmissionError>>;
    fn fail_capacity(&mut self) -> CanonicalEncodeError<Self::AdmissionError>;
    fn fail_allocation(&mut self) -> CanonicalEncodeError<Self::AdmissionError>;
    fn admit(
        &mut self,
        charge: CanonicalEncodingCharge,
    ) -> Result<(), CanonicalEncodeError<Self::AdmissionError>>;
    fn put(&mut self, bytes: &[u8]) -> Result<(), CanonicalEncodeError<Self::AdmissionError>>;
    fn buffered(&mut self, bytes: usize) -> Result<(), CanonicalEncodeError<Self::AdmissionError>>;
}

/// Reserve a named Vec backing before it grows. Work covers only initialized
/// slots that a growth may move; scratch covers the full new backing because
/// it coexists with the old backing until reallocation completes.
pub(super) fn reserve<T, S: Sink + ?Sized>(
    sink: &mut S,
    value: &mut Vec<T>,
    additional: usize,
) -> Result<(), CanonicalEncodeError<S::AdmissionError>> {
    let required = value
        .len()
        .checked_add(additional)
        .ok_or_else(|| sink.fail_capacity())?;
    if required <= value.capacity() {
        return Ok(());
    }
    let target = value
        .capacity()
        .max(4)
        .checked_mul(2)
        .map(|grown| grown.max(required))
        .ok_or_else(|| sink.fail_capacity())?;
    let bytes = target
        .checked_mul(size_of::<T>())
        .ok_or_else(|| sink.fail_capacity())?;
    if sink.tracks_work() {
        let moved = u64::try_from(value.len()).map_err(|_| sink.fail_capacity())?;
        sink.admit(CanonicalEncodingCharge::Work(moved))?;
        let bytes = u64::try_from(bytes).map_err(|_| sink.fail_capacity())?;
        sink.admit(CanonicalEncodingCharge::Scratch(bytes))?;
    } else {
        sink.poll()?;
    }
    value
        .try_reserve_exact(target - value.len())
        .map_err(|_| sink.fail_allocation())
}

impl Sink for Vec<u8> {
    type AdmissionError = Infallible;
    fn poll(&mut self) -> Result<(), CanonicalEncodeError<Infallible>> {
        Ok(())
    }
    fn ensure_active(&self) -> Result<(), CanonicalEncodeError<Infallible>> {
        Ok(())
    }
    fn fail_capacity(&mut self) -> CanonicalEncodeError<Infallible> {
        CanonicalEncodeError::CapacityOverflow
    }
    fn fail_allocation(&mut self) -> CanonicalEncodeError<Infallible> {
        CanonicalEncodeError::Allocation
    }
    fn admit(
        &mut self,
        _: CanonicalEncodingCharge,
    ) -> Result<(), CanonicalEncodeError<Self::AdmissionError>> {
        Ok(())
    }
    fn put(&mut self, bytes: &[u8]) -> Result<(), CanonicalEncodeError<Self::AdmissionError>> {
        self.extend_from_slice(bytes);
        Ok(())
    }
    fn buffered(&mut self, _: usize) -> Result<(), CanonicalEncodeError<Self::AdmissionError>> {
        Ok(())
    }
}

/// Feeds SHA-256 while preserving the established canonical work counters.
pub struct HashingSink<A: EncodingAdmission = NoAdmission> {
    digest: Sha256,
    hashed_bytes: usize,
    buffered_bytes: usize,
    admission: A,
    fatal: Option<OwnerFatal>,
    captured: Option<Vec<u8>>,
}

#[derive(Clone, Copy)]
enum OwnerFatal {
    CapacityOverflow,
    Allocation,
}

impl OwnerFatal {
    fn error<E: Debug>(self) -> CanonicalEncodeError<E> {
        match self {
            Self::CapacityOverflow => CanonicalEncodeError::CapacityOverflow,
            Self::Allocation => CanonicalEncodeError::Allocation,
        }
    }
}

impl HashingSink<NoAdmission> {
    pub fn new() -> Self {
        Self::with_admission(NoAdmission)
    }
}

impl<A: EncodingAdmission> HashingSink<A> {
    pub fn with_admission(admission: A) -> Self {
        Self {
            digest: Sha256::new(),
            hashed_bytes: 0,
            buffered_bytes: 0,
            admission,
            fatal: None,
            captured: None,
        }
    }

    pub const fn hashed_bytes(&self) -> usize {
        self.hashed_bytes
    }

    pub fn capturing(mut self, enabled: bool) -> Self {
        self.captured = enabled.then(Vec::new);
        self
    }

    pub fn take_capture(&mut self) -> Option<Vec<u8>> {
        self.captured.take()
    }

    pub const fn buffered_bytes(&self) -> usize {
        self.buffered_bytes
    }

    pub fn take_terminal_failure(&mut self) -> Option<CanonicalEncodeError<A::Error>> {
        self.admission
            .take_failure()
            .map(CanonicalEncodeError::Admission)
            .or_else(|| self.fatal.map(OwnerFatal::error))
    }

    pub fn finish(mut self) -> Result<[u8; 32], CanonicalEncodeError<A::Error>> {
        if let Some(error) = self.take_terminal_failure() {
            return Err(error);
        }
        let padding = if self.tracks_work() {
            let total_blocks = self
                .hashed_bytes
                .checked_add(9 + 63)
                .ok_or_else(|| self.fail_capacity())?
                / 64;
            let pending_blocks = total_blocks - self.hashed_bytes / 64;
            let pending_units = u64::try_from(pending_blocks).map_err(|_| self.fail_capacity())?;
            self.admit(CanonicalEncodingCharge::Work(pending_units))
        } else {
            self.poll()
        };
        if let Some(error) = self.take_terminal_failure() {
            return Err(error);
        }
        padding?;
        Ok(self.digest.finalize().into())
    }
}

impl<A: EncodingAdmission> Sink for HashingSink<A> {
    type AdmissionError = A::Error;

    fn tracks_work(&self) -> bool {
        self.admission.tracks_work()
    }
    fn poll(&mut self) -> Result<(), CanonicalEncodeError<A::Error>> {
        self.ensure_active()?;
        self.admission.poll()
    }
    fn ensure_active(&self) -> Result<(), CanonicalEncodeError<A::Error>> {
        if self.admission.has_failure() {
            Err(CanonicalEncodeError::AdmissionDeferred)
        } else if let Some(fatal) = self.fatal {
            Err(fatal.error())
        } else {
            Ok(())
        }
    }
    fn fail_capacity(&mut self) -> CanonicalEncodeError<A::Error> {
        if let Err(error) = self.ensure_active() {
            return error;
        }
        self.fatal = Some(OwnerFatal::CapacityOverflow);
        CanonicalEncodeError::CapacityOverflow
    }
    fn fail_allocation(&mut self) -> CanonicalEncodeError<A::Error> {
        if let Err(error) = self.ensure_active() {
            return error;
        }
        self.fatal = Some(OwnerFatal::Allocation);
        CanonicalEncodeError::Allocation
    }
    fn admit(
        &mut self,
        charge: CanonicalEncodingCharge,
    ) -> Result<(), CanonicalEncodeError<Self::AdmissionError>> {
        self.ensure_active()?;
        self.admission.admit(charge)
    }

    fn put(&mut self, bytes: &[u8]) -> Result<(), CanonicalEncodeError<Self::AdmissionError>> {
        self.ensure_active()?;
        let next = self
            .hashed_bytes
            .checked_add(bytes.len())
            .ok_or_else(|| self.fail_capacity())?;
        if self.tracks_work() {
            let blocks = next / 64 - self.hashed_bytes / 64;
            let units = bytes
                .len()
                .checked_add(blocks)
                .and_then(|units| u64::try_from(units).ok())
                .ok_or_else(|| self.fail_capacity())?;
            self.admit(CanonicalEncodingCharge::Work(units))?;
        } else {
            self.poll()?;
        }
        if let Some(mut captured) = self.captured.take() {
            reserve(self, &mut captured, bytes.len())?;
            if self.tracks_work() {
                self.admit(CanonicalEncodingCharge::Work(bytes.len() as u64))?;
            }
            captured.extend_from_slice(bytes);
            self.buffered(bytes.len())?;
            self.captured = Some(captured);
        }
        Digest::update(&mut self.digest, bytes);
        self.hashed_bytes = next;
        Ok(())
    }

    fn buffered(&mut self, bytes: usize) -> Result<(), CanonicalEncodeError<Self::AdmissionError>> {
        self.ensure_active()?;
        if self.tracks_work() {
            self.buffered_bytes = self
                .buffered_bytes
                .checked_add(bytes)
                .ok_or_else(|| self.fail_capacity())?;
        }
        Ok(())
    }
}
