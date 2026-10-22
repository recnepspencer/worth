//! Value validation and actual backing policy for the one canonical reducer.
use std::{alloc::Layout, mem::size_of};

use super::{memory::node_bytes, Link, Node, ReductionDenial, ReductionRunStop};
use crate::{CanonicalBits, ChargedBytes, ExecutionMemoryReservation, ExecutionResourceLease};

pub(super) struct CanonicalBytes {
    bytes: Vec<u8>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl PartialEq for CanonicalBytes {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

pub(super) trait ValueStorage<T>: Copy {
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
    fn node<E>(
        &self,
        value: &T,
        aggregate: &T,
        left: &Link<T>,
        right: &Link<T>,
    ) -> Result<(u64, Option<ExecutionMemoryReservation>), ReductionRunStop<E>>;
}

#[derive(Clone, Copy)]
pub(super) struct BoundedValues(pub(super) u64);
impl<T: CanonicalBits + ChargedBytes> ValueStorage<T> for BoundedValues {
    fn validate<E>(&self, value: &T) -> Result<(), ReductionRunStop<E>> {
        let bytes = u64::try_from(size_of::<T>())
            .ok()
            .and_then(|inline| inline.checked_add(value.additional_charged_bytes()))
            .ok_or(ReductionRunStop::ResultCapacityExceeded)?;
        if bytes > self.0 {
            return Err(ReductionRunStop::ResultCapacityExceeded);
        }
        self.canonical(value).map(|_| ())
    }
    fn canonical<E>(&self, value: &T) -> Result<CanonicalBytes, ReductionRunStop<E>> {
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
    fn node<E>(
        &self,
        value: &T,
        aggregate: &T,
        left: &Link<T>,
        right: &Link<T>,
    ) -> Result<(u64, Option<ExecutionMemoryReservation>), ReductionRunStop<E>> {
        Ok((
            node_bytes(value, aggregate, left, right)
                .ok_or(ReductionRunStop::ResultCapacityExceeded)?,
            None,
        ))
    }
}

#[derive(Clone, Copy)]
pub(super) struct NativeValues<'scope, 'authority> {
    pub(super) lease: Option<&'scope ExecutionResourceLease<'authority>>,
}
impl<T: CanonicalBits> ValueStorage<T> for NativeValues<'_, '_> {
    fn backing<E>(
        &self,
        layout: Layout,
    ) -> Result<Option<ExecutionMemoryReservation>, ReductionRunStop<E>> {
        if layout.size() == 0 {
            return Ok(None);
        }
        reserve(self.lease, layout.size()).map(Some)
    }
    fn canonical<E>(&self, value: &T) -> Result<CanonicalBytes, ReductionRunStop<E>> {
        let length = length(value)?;
        let layout = Layout::array::<u8>(length)
            .map_err(|_| physical(crate::ExecutionPhysicalAllocationDenialKind::Layout))?;
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
    ) -> Result<(u64, Option<ExecutionMemoryReservation>), ReductionRunStop<E>> {
        let layout = Layout::new::<[usize; 2]>()
            .extend(Layout::new::<Node<T>>())
            .map_err(|_| physical(crate::ExecutionPhysicalAllocationDenialKind::Layout))?
            .0
            .pad_to_align();
        let memory = reserve(self.lease, layout.size())?;
        let bytes = memory
            .bytes()
            .checked_add(left.as_ref().map_or(0, |node| node.retained_bytes))
            .and_then(|bytes| {
                bytes.checked_add(right.as_ref().map_or(0, |node| node.retained_bytes))
            })
            .ok_or_else(|| physical(crate::ExecutionPhysicalAllocationDenialKind::Layout))?;
        Ok((bytes, Some(memory)))
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
