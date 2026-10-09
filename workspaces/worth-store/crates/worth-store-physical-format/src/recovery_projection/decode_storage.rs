//! Allocation admission for owned redo decoding. Admission grants no replay authority.

use std::{collections::TryReserveError, convert::Infallible};

use super::PhysicalRecoveryProjectionDenial;
use crate::CanonicalRedoWireDenial;

/// The caller owns every admitted charge until the decoded values and decoder
/// scratch have been disposed. Requests include temporarily overlapping backing;
/// implementations may conservatively retain all requests for that lifetime.
pub trait PhysicalRecoveryDecodeStorage {
    type Denial;
    fn admit_allocation(&mut self, bytes: u64) -> Result<(), Self::Denial>;
}

#[derive(Debug)]
pub enum PhysicalRecoveryDecodeFailure<D> {
    Canonical(CanonicalRedoWireDenial),
    Projection(PhysicalRecoveryProjectionDenial),
    Allocation(D),
    AllocationFailed {
        requested: u64,
        cause: TryReserveError,
    },
    AllocatorExceededReservation {
        requested: u64,
        actual: u64,
    },
    SizeOverflow,
}

impl<D> From<PhysicalRecoveryProjectionDenial> for PhysicalRecoveryDecodeFailure<D> {
    fn from(denial: PhysicalRecoveryProjectionDenial) -> Self {
        Self::Projection(denial)
    }
}

impl<D> From<CanonicalRedoWireDenial> for PhysicalRecoveryDecodeFailure<D> {
    fn from(denial: CanonicalRedoWireDenial) -> Self {
        Self::Canonical(denial)
    }
}

pub(crate) struct UnrestrictedDecodeStorage;

impl PhysicalRecoveryDecodeStorage for UnrestrictedDecodeStorage {
    type Denial = Infallible;
    fn admit_allocation(&mut self, _: u64) -> Result<(), Infallible> {
        Ok(())
    }
}

pub(crate) fn reserve_vec<T, S: PhysicalRecoveryDecodeStorage>(
    count: usize,
    storage: &mut S,
) -> Result<Vec<T>, PhysicalRecoveryDecodeFailure<S::Denial>> {
    let bytes = count
        .checked_mul(std::mem::size_of::<T>())
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(PhysicalRecoveryDecodeFailure::SizeOverflow)?;
    storage
        .admit_allocation(bytes)
        .map_err(PhysicalRecoveryDecodeFailure::Allocation)?;
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|cause| {
        PhysicalRecoveryDecodeFailure::AllocationFailed {
            requested: bytes,
            cause,
        }
    })?;
    if std::mem::size_of::<T>() != 0 && values.capacity() != count {
        let actual = (values.capacity() as u64)
            .checked_mul(std::mem::size_of::<T>() as u64)
            .ok_or(PhysicalRecoveryDecodeFailure::SizeOverflow)?;
        return Err(
            PhysicalRecoveryDecodeFailure::AllocatorExceededReservation {
                requested: bytes,
                actual,
            },
        );
    }
    Ok(values)
}

pub(crate) fn copy_box<S: PhysicalRecoveryDecodeStorage>(
    bytes: &[u8],
    storage: &mut S,
) -> Result<Box<[u8]>, PhysicalRecoveryDecodeFailure<S::Denial>> {
    let mut owned = reserve_vec(bytes.len(), storage)?;
    owned.extend_from_slice(bytes);
    Ok(owned.into_boxed_slice())
}

pub(crate) fn projection_denial(
    failure: PhysicalRecoveryDecodeFailure<Infallible>,
) -> PhysicalRecoveryProjectionDenial {
    match failure {
        PhysicalRecoveryDecodeFailure::Projection(denial) => denial,
        PhysicalRecoveryDecodeFailure::Allocation(never) => match never {},
        _ => PhysicalRecoveryProjectionDenial::EntryLimit,
    }
}

pub(crate) fn canonical_denial(
    failure: PhysicalRecoveryDecodeFailure<Infallible>,
) -> CanonicalRedoWireDenial {
    match failure {
        PhysicalRecoveryDecodeFailure::Canonical(denial) => denial,
        PhysicalRecoveryDecodeFailure::Projection(PhysicalRecoveryProjectionDenial::EntryLimit) => {
            CanonicalRedoWireDenial::ProjectionEntryLimit
        }
        PhysicalRecoveryDecodeFailure::Projection(
            PhysicalRecoveryProjectionDenial::UnsupportedVersion(version),
        ) => CanonicalRedoWireDenial::UnsupportedRecoveryProjectionVersion(version),
        PhysicalRecoveryDecodeFailure::Projection(_) => {
            CanonicalRedoWireDenial::InvalidRecoveryProjection
        }
        PhysicalRecoveryDecodeFailure::Allocation(never) => match never {},
        _ => CanonicalRedoWireDenial::CounterOverflow,
    }
}
