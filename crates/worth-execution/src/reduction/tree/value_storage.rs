//! Value validation and actual backing policy for the one canonical reducer.
use std::{alloc::Layout, mem::size_of, sync::Arc};

use super::{memory::node_bytes, Link, Node, ReductionDenial, ReductionRunStop};
use crate::{CanonicalBits, ChargedBytes, ExecutionMemoryReservation, ExecutionResourceLease};

/// Only the canonical storage owner constructs a canonical proof. Native slots
/// use their separate immutable receipt/summary owner, never a fake encoding.
#[derive(Clone)]
pub(super) struct Admitted<T> {
    pub(super) value: T,
    pub(super) canonical: Option<Arc<CanonicalBytes>>,
}
impl<T> std::ops::Deref for Admitted<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}
pub(super) struct CanonicalBytes {
    bytes: Vec<u8>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl CanonicalBytes {
    pub(super) fn object_layout() -> Layout {
        Layout::new::<[usize; 2]>()
            .extend(Layout::new::<Self>())
            .expect("canonical proof object layout is representable")
            .0
            .pad_to_align()
    }
    pub(super) fn retained_bytes(&self) -> u64 {
        self.bytes.capacity() as u64 + Self::object_layout().size() as u64
    }
}
impl PartialEq for CanonicalBytes {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

pub(super) trait ValueStorage<T>: Copy {
    fn has_accounting(&self) -> bool {
        true
    }
    fn backing<E>(
        &self,
        _layout: Layout,
    ) -> Result<Option<ExecutionMemoryReservation>, ReductionRunStop<E>> {
        Ok(None)
    }
    fn buffer<U, E>(
        &self,
        length: usize,
    ) -> Result<(Vec<U>, Option<ExecutionMemoryReservation>), ReductionRunStop<E>> {
        let layout = Layout::array::<U>(length)
            .map_err(|_| physical(crate::ExecutionPhysicalAllocationDenialKind::Layout))?;
        let memory = self.backing(layout)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(length)
            .map_err(|_| physical(crate::ExecutionPhysicalAllocationDenialKind::Allocator))?;
        Ok((values, memory))
    }
    fn validate<E>(&self, value: &T) -> Result<(), ReductionRunStop<E>> {
        self.canonical(value).map(|_| ())
    }
    fn canonical<E>(&self, value: &T) -> Result<CanonicalBytes, ReductionRunStop<E>>;
    fn check_proof<E>(
        &self,
        _: &T,
        _: &CanonicalBytes,
        _: bool,
    ) -> Result<(), ReductionRunStop<E>> {
        Ok(())
    }
    fn node<E>(
        &self,
        value: &T,
        aggregate: &T,
        left: &Link<T>,
        right: &Link<T>,
    ) -> Result<(Option<u64>, Option<ExecutionMemoryReservation>), ReductionRunStop<E>>;
}

#[derive(Clone, Copy)]
pub(super) struct BoundedValues(pub(super) u64);
impl<T: CanonicalBits + ChargedBytes> ValueStorage<T> for BoundedValues {
    fn canonical<E>(&self, value: &T) -> Result<CanonicalBytes, ReductionRunStop<E>> {
        let payload_bytes = (size_of::<T>() as u64)
            .checked_add(value.additional_charged_bytes())
            .ok_or(ReductionRunStop::ResultCapacityExceeded)?;
        if payload_bytes > self.0 {
            return Err(ReductionRunStop::ResultCapacityExceeded);
        }
        let length = length(value)?;
        if u64::try_from(length)
            .ok()
            .is_none_or(|bytes| bytes > self.0)
        {
            return Err(ReductionRunStop::ResultCapacityExceeded);
        }
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length)
            .map_err(|_| ReductionRunStop::ResultCapacityExceeded)?;
        encode(value, length, &mut bytes)?;
        Ok(CanonicalBytes {
            bytes,
            _memory: None,
        })
    }
    fn check_proof<E>(
        &self,
        value: &T,
        proof: &CanonicalBytes,
        payload: bool,
    ) -> Result<(), ReductionRunStop<E>> {
        if proof.bytes.len() as u64 > self.0
            || (payload
                && (size_of::<T>() as u64)
                    .checked_add(value.additional_charged_bytes())
                    .is_none_or(|bytes| bytes > self.0))
        {
            return Err(ReductionRunStop::ResultCapacityExceeded);
        }
        Ok(())
    }
    fn node<E>(
        &self,
        value: &T,
        aggregate: &T,
        left: &Link<T>,
        right: &Link<T>,
    ) -> Result<(Option<u64>, Option<ExecutionMemoryReservation>), ReductionRunStop<E>> {
        Ok((
            Some(
                node_bytes(value, aggregate, left, right)
                    .ok_or(ReductionRunStop::ResultCapacityExceeded)?,
            ),
            None,
        ))
    }
}

#[derive(Clone, Copy)]
pub(super) struct NativeValues<'scope, 'authority> {
    pub(super) lease: Option<&'scope ExecutionResourceLease<'authority>>,
}
impl<T: CanonicalBits> ValueStorage<T> for NativeValues<'_, '_> {
    fn has_accounting(&self) -> bool {
        crate::backend::RunLimits::native_accounting(self.lease)
    }
    fn backing<E>(
        &self,
        layout: Layout,
    ) -> Result<Option<ExecutionMemoryReservation>, ReductionRunStop<E>> {
        crate::backend::RunLimits::check_native_scope(self.lease)
            .map_err(|cause| ReductionRunStop::Denial(ReductionDenial::Admission(cause)))?;
        if !<Self as ValueStorage<T>>::has_accounting(self) || layout.size() == 0 {
            return Ok(None);
        }
        reserve(self.lease, layout.size()).map(Some)
    }
    fn canonical<E>(&self, value: &T) -> Result<CanonicalBytes, ReductionRunStop<E>> {
        let length = length(value)?;
        let layout = Layout::array::<u8>(length)
            .ok()
            .and_then(|buffer| {
                CanonicalBytes::object_layout()
                    .extend(buffer)
                    .ok()
                    .map(|v| v.0)
            })
            .ok_or_else(|| physical(crate::ExecutionPhysicalAllocationDenialKind::Layout))?;
        let memory = <Self as ValueStorage<T>>::backing(self, layout)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length)
            .map_err(|_| physical(crate::ExecutionPhysicalAllocationDenialKind::Allocator))?;
        encode(value, length, &mut bytes)?;
        Ok(CanonicalBytes {
            bytes,
            _memory: memory,
        })
    }
    fn node<E>(
        &self,
        _: &T,
        _: &T,
        left: &Link<T>,
        right: &Link<T>,
    ) -> Result<(Option<u64>, Option<ExecutionMemoryReservation>), ReductionRunStop<E>> {
        let layout = Layout::new::<[usize; 2]>()
            .extend(Layout::new::<Node<T>>())
            .map_err(|_| physical(crate::ExecutionPhysicalAllocationDenialKind::Layout))?
            .0
            .pad_to_align();
        let Some(memory) = <Self as ValueStorage<T>>::backing(self, layout)? else {
            return Ok((None, None));
        };
        let child_bytes = |link: &Link<T>| match link {
            None => Ok(0),
            Some(node) => node.retained_bytes.ok_or(ReductionRunStop::Denial(
                ReductionDenial::AccountingUnavailable,
            )),
        };
        let left_bytes = child_bytes(left)?;
        let right_bytes = child_bytes(right)?;
        let bytes = memory
            .bytes()
            .checked_add(left_bytes)
            .and_then(|bytes| bytes.checked_add(right_bytes))
            .ok_or_else(|| physical(crate::ExecutionPhysicalAllocationDenialKind::Layout))?;
        Ok((Some(bytes), Some(memory)))
    }
}

fn reserve<E>(
    lease: Option<&ExecutionResourceLease<'_>>,
    bytes: usize,
) -> Result<ExecutionMemoryReservation, ReductionRunStop<E>> {
    let bytes = u64::try_from(bytes)
        .map_err(|_| physical(crate::ExecutionPhysicalAllocationDenialKind::Layout))?;
    ExecutionMemoryReservation::reserve_in_scope(lease, bytes)
        .map_err(|cause| ReductionRunStop::Denial(ReductionDenial::Admission(cause)))
}

fn physical<E>(kind: crate::ExecutionPhysicalAllocationDenialKind) -> ReductionRunStop<E> {
    ReductionRunStop::Denial(ReductionDenial::Allocation(kind))
}
fn length<T: CanonicalBits, E>(value: &T) -> Result<usize, ReductionRunStop<E>> {
    value.canonical_len().ok_or(ReductionRunStop::Denial(
        ReductionDenial::InvalidCanonicalEncoding,
    ))
}
fn encode<T: CanonicalBits, E>(
    value: &T,
    length: usize,
    bytes: &mut Vec<u8>,
) -> Result<(), ReductionRunStop<E>> {
    let mut valid = true;
    let complete = value.visit_canonical_bits(&mut |chunk| {
        if chunk.len() > length.saturating_sub(bytes.len()) {
            valid = false;
            return false;
        }
        bytes.extend_from_slice(chunk);
        true
    });
    if valid && complete && bytes.len() == length {
        Ok(())
    } else {
        Err(ReductionRunStop::Denial(
            ReductionDenial::InvalidCanonicalEncoding,
        ))
    }
}
