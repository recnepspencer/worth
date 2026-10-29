use std::sync::Arc;

use super::{
    allocation::ProcessPhysicalFrameAllocator, PhysicalFrameAllocator, PhysicalFrameFaultError,
    PhysicalFrameLoadTerminalKind, PhysicalFrameLoadingIdentity,
};
use crate::physical_residency::pool::PoolInner;
use crate::physical_residency::{
    PhysicalBoundedFrameKey, PhysicalFrameLease, PhysicalResidencyDenial,
};

/// Move-owned authority for length discovery and byte loading of one bounded fault.
#[derive(Debug)]
pub struct PhysicalBoundedFrameFaultOwner {
    pub(crate) owner: Arc<PoolInner>,
    pub(crate) key: PhysicalBoundedFrameKey,
    pub(crate) identity: PhysicalFrameLoadingIdentity,
    pub(crate) scope: crate::PhysicalOperationAllocationScope,
    pub(crate) armed: bool,
}

impl PhysicalBoundedFrameFaultOwner {
    pub const fn loading_identity(&self) -> PhysicalFrameLoadingIdentity {
        self.identity
    }

    pub fn load<E, L, F>(
        self,
        discover_length: L,
        fill: F,
    ) -> Result<PhysicalFrameLease, PhysicalFrameFaultError<E>>
    where
        L: FnOnce(u32) -> Result<u32, E>,
        F: FnOnce(&mut [u8]) -> Result<(), E>,
    {
        self.load_core(&ProcessPhysicalFrameAllocator, discover_length, |target| {
            fill(target).map(|()| None)
        })
    }

    #[cfg(test)]
    pub(crate) fn load_with_allocator<E, L, F>(
        self,
        allocator: &dyn PhysicalFrameAllocator,
        discover_length: L,
        fill: F,
    ) -> Result<PhysicalFrameLease, PhysicalFrameFaultError<E>>
    where
        L: FnOnce(u32) -> Result<u32, E>,
        F: FnOnce(&mut [u8]) -> Result<(), E>,
    {
        self.load_core(allocator, discover_length, |target| {
            fill(target).map(|()| None)
        })
    }

    pub fn load_observed<E, L, F>(
        self,
        discover_length: L,
        fill: F,
    ) -> Result<PhysicalFrameLease, PhysicalFrameFaultError<E>>
    where
        L: FnOnce(u32) -> Result<u32, E>,
        F: FnOnce(&mut [u8]) -> Result<Option<crate::PhysicalResidencyAllocationOperation>, E>,
    {
        self.load_core(&ProcessPhysicalFrameAllocator, discover_length, fill)
    }

    fn load_core<E, L, F>(
        mut self,
        allocator: &dyn PhysicalFrameAllocator,
        discover_length: L,
        fill: F,
    ) -> Result<PhysicalFrameLease, PhysicalFrameFaultError<E>>
    where
        L: FnOnce(u32) -> Result<u32, E>,
        F: FnOnce(&mut [u8]) -> Result<Option<crate::PhysicalResidencyAllocationOperation>, E>,
    {
        let mut discover_failure = None;
        let mut fill_failure = None;
        let mut discover_length = Some(discover_length);
        let mut fill = Some(fill);
        match self.load_bounded_frame(
            allocator,
            &mut |limit| match discover_length
                .take()
                .expect("bounded length discovery runs once")(limit)
            {
                Ok(length) => Ok(length),
                Err(error) => {
                    discover_failure = Some(error);
                    Err(())
                }
            },
            &mut |target| match fill.take().expect("bounded frame fill runs once")(target) {
                Ok(operation) => Ok(operation),
                Err(error) => {
                    fill_failure = Some(error);
                    Err(())
                }
            },
        ) {
            Ok(lease) => Ok(lease),
            Err(BoundedLoadStop::Residency { terminal, denial }) => {
                Err(PhysicalFrameFaultError::Residency { terminal, denial })
            }
            Err(BoundedLoadStop::Source { terminal }) => Err(PhysicalFrameFaultError::Source {
                terminal,
                failure: discover_failure
                    .or(fill_failure)
                    .expect("bounded source failure is stored"),
            }),
        }
    }

    /// One compiled length, allocation, and publish path for every bounded fill.
    #[inline(never)]
    fn load_bounded_frame(
        &mut self,
        allocator: &dyn PhysicalFrameAllocator,
        discover_length: &mut dyn FnMut(u32) -> Result<u32, ()>,
        fill: &mut dyn FnMut(
            &mut [u8],
        )
            -> Result<Option<crate::PhysicalResidencyAllocationOperation>, ()>,
    ) -> Result<PhysicalFrameLease, BoundedLoadStop> {
        self.owner.record_source_load();
        let length = match discover_length(self.key.limit()) {
            Ok(length) if length > 0 && length <= self.key.limit() => length,
            Ok(_) => return Err(self.reject_invalid_length()),
            Err(()) => return Err(self.reject_source()),
        };
        let mut bytes = match allocator.allocate(length as usize) {
            Ok(bytes) => bytes,
            Err(()) => {
                return Err(self.reject_allocation(PhysicalResidencyDenial::AllocationFailed))
            }
        };
        let actual = bytes.capacity() as u64;
        let requested = u64::from(self.key.limit());
        if actual > requested {
            return Err(self.reject_allocation(
                PhysicalResidencyDenial::AllocatorExceededReservation { requested, actual },
            ));
        }
        let operation = fill(bytes.as_mut_slice()).map_err(|()| self.reject_source())?;
        self.owner.actualize_allocation(
            crate::physical_residency::PhysicalResidencyAllocationActualization::new(
                crate::PhysicalResidencyDimension::ResidentBytes,
                self.scope,
                crate::physical_residency::PhysicalResidencyRequestedAllocationUnits::new(
                    requested,
                ),
                crate::physical_residency::PhysicalResidencyActualAllocationUnits::new(actual),
            )
            .with_operation(operation),
        );
        match self.owner.finish_bounded_loading(
            self.key,
            self.identity,
            length,
            bytes.into_resident(),
        ) {
            Ok(lease) => {
                self.armed = false;
                Ok(lease)
            }
            Err((denial, terminal)) => {
                self.armed = false;
                Err(BoundedLoadStop::Residency { terminal, denial })
            }
        }
    }

    fn reject_invalid_length(&mut self) -> BoundedLoadStop {
        let terminal = self.owner.fail_bounded_loading(
            self.key,
            self.identity,
            PhysicalFrameLoadTerminalKind::SourcePreparationFailed,
        );
        self.armed = false;
        BoundedLoadStop::Residency {
            terminal,
            denial: self
                .owner
                .record_denial(PhysicalResidencyDenial::FrameLengthMismatch),
        }
    }

    fn reject_allocation(&mut self, denial: PhysicalResidencyDenial) -> BoundedLoadStop {
        let terminal = self.owner.fail_bounded_loading(
            self.key,
            self.identity,
            PhysicalFrameLoadTerminalKind::AllocationFailed,
        );
        self.armed = false;
        BoundedLoadStop::Residency {
            terminal,
            denial: self.owner.record_denial(denial),
        }
    }

    fn reject_source(&mut self) -> BoundedLoadStop {
        let terminal = self.owner.fail_bounded_loading(
            self.key,
            self.identity,
            PhysicalFrameLoadTerminalKind::SourceExecutionFailed,
        );
        self.armed = false;
        BoundedLoadStop::Source { terminal }
    }
}

enum BoundedLoadStop {
    Residency {
        terminal: super::PhysicalFrameLoadTerminal,
        denial: PhysicalResidencyDenial,
    },
    Source {
        terminal: super::PhysicalFrameLoadTerminal,
    },
}

impl Drop for PhysicalBoundedFrameFaultOwner {
    fn drop(&mut self) {
        if self.armed {
            self.owner.abandon_bounded_loading(self.key, self.identity);
        }
    }
}
