use std::sync::Arc;

use super::{
    allocation::ProcessPhysicalFrameAllocator, PhysicalFrameAllocator, PhysicalFrameLoadTerminal,
    PhysicalFrameLoadTerminalKind, PhysicalFrameLoadingIdentity,
};
use crate::physical_residency::pool::PoolInner;
use crate::physical_residency::{PhysicalFrameKey, PhysicalFrameLease, PhysicalResidencyDenial};

/// Move-owned authority for the sole source load of one admitted frame fault.
#[derive(Debug)]
pub struct PhysicalFrameFaultOwner {
    pub(crate) owner: Arc<PoolInner>,
    pub(crate) key: PhysicalFrameKey,
    pub(crate) identity: PhysicalFrameLoadingIdentity,
    pub(crate) scope: crate::PhysicalOperationAllocationScope,
    pub(crate) armed: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PhysicalFrameFaultError<E> {
    Residency {
        terminal: PhysicalFrameLoadTerminal,
        denial: PhysicalResidencyDenial,
    },
    Source {
        terminal: PhysicalFrameLoadTerminal,
        failure: E,
    },
}

impl PhysicalFrameFaultOwner {
    pub const fn loading_identity(&self) -> PhysicalFrameLoadingIdentity {
        self.identity
    }

    pub fn reject_before_source(mut self) -> PhysicalFrameLoadTerminal {
        let terminal = self.owner.fail_loading(
            self.key,
            self.identity,
            PhysicalFrameLoadTerminalKind::SourcePreparationFailed,
        );
        self.armed = false;
        terminal
    }

    pub fn load<E, F>(self, fill: F) -> Result<PhysicalFrameLease, PhysicalFrameFaultError<E>>
    where
        F: FnOnce(&mut [u8]) -> Result<(), E>,
    {
        self.load_with_allocator_and_operation(&ProcessPhysicalFrameAllocator, None, fill)
    }

    pub fn load_observed<E, F>(
        self,
        operation: Option<crate::PhysicalResidencyAllocationOperation>,
        fill: F,
    ) -> Result<PhysicalFrameLease, PhysicalFrameFaultError<E>>
    where
        F: FnOnce(&mut [u8]) -> Result<(), E>,
    {
        self.load_with_allocator_and_operation(&ProcessPhysicalFrameAllocator, operation, fill)
    }

    #[cfg(test)]
    pub(crate) fn load_with_allocator<E, F>(
        self,
        allocator: &dyn PhysicalFrameAllocator,
        fill: F,
    ) -> Result<PhysicalFrameLease, PhysicalFrameFaultError<E>>
    where
        F: FnOnce(&mut [u8]) -> Result<(), E>,
    {
        self.load_with_allocator_and_operation(allocator, None, fill)
    }

    fn load_with_allocator_and_operation<E, F>(
        mut self,
        allocator: &dyn PhysicalFrameAllocator,
        operation: Option<crate::PhysicalResidencyAllocationOperation>,
        fill: F,
    ) -> Result<PhysicalFrameLease, PhysicalFrameFaultError<E>>
    where
        F: FnOnce(&mut [u8]) -> Result<(), E>,
    {
        let mut failure = None;
        let mut fill = Some(fill);
        match self.load_filled_frame(allocator, operation, &mut |bytes| match fill
            .take()
            .expect("frame fill runs once")(
            bytes
        ) {
            Ok(()) => true,
            Err(error) => {
                failure = Some(error);
                false
            }
        }) {
            Ok(lease) => Ok(lease),
            Err(FrameLoadStop::Residency { terminal, denial }) => {
                Err(PhysicalFrameFaultError::Residency { terminal, denial })
            }
            Err(FrameLoadStop::Source { terminal }) => Err(PhysicalFrameFaultError::Source {
                terminal,
                failure: failure.expect("frame source failure is stored"),
            }),
        }
    }

    /// One compiled admission, allocation, and publish path for every fill closure.
    #[inline(never)]
    fn load_filled_frame(
        &mut self,
        allocator: &dyn PhysicalFrameAllocator,
        operation: Option<crate::PhysicalResidencyAllocationOperation>,
        fill: &mut dyn FnMut(&mut [u8]) -> bool,
    ) -> Result<PhysicalFrameLease, FrameLoadStop> {
        let length = self.key.coordinate().length() as usize;
        let mut bytes = match allocator.allocate(length) {
            Ok(bytes) => bytes,
            Err(()) => {
                let terminal = self.owner.fail_loading(
                    self.key,
                    self.identity,
                    PhysicalFrameLoadTerminalKind::AllocationFailed,
                );
                self.armed = false;
                return Err(FrameLoadStop::Residency {
                    terminal,
                    denial: self
                        .owner
                        .record_denial(PhysicalResidencyDenial::AllocationFailed),
                });
            }
        };
        let actual = bytes.capacity() as u64;
        let requested = length as u64;
        if actual > requested {
            let terminal = self.owner.fail_loading(
                self.key,
                self.identity,
                PhysicalFrameLoadTerminalKind::AllocationFailed,
            );
            self.armed = false;
            return Err(FrameLoadStop::Residency {
                terminal,
                denial: self.owner.record_denial(
                    PhysicalResidencyDenial::AllocatorExceededReservation { requested, actual },
                ),
            });
        }
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
        self.owner.record_source_load();
        if !fill(bytes.as_mut_slice()) {
            let terminal = self.owner.fail_loading(
                self.key,
                self.identity,
                PhysicalFrameLoadTerminalKind::SourceExecutionFailed,
            );
            self.armed = false;
            return Err(FrameLoadStop::Source { terminal });
        }
        match self
            .owner
            .finish_loading(self.key, self.identity, bytes.into_resident())
        {
            Ok(lease) => {
                self.armed = false;
                Ok(lease)
            }
            Err((denial, terminal)) => {
                self.armed = false;
                Err(FrameLoadStop::Residency { terminal, denial })
            }
        }
    }
}

enum FrameLoadStop {
    Residency {
        terminal: PhysicalFrameLoadTerminal,
        denial: PhysicalResidencyDenial,
    },
    Source {
        terminal: PhysicalFrameLoadTerminal,
    },
}

impl Drop for PhysicalFrameFaultOwner {
    fn drop(&mut self) {
        if self.armed {
            self.owner.abandon_loading(self.key, self.identity);
        }
    }
}
