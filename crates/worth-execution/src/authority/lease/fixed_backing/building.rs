use super::super::ExecutionLeaseStatus;
use super::policy::check_status;
use super::{
    ExecutionAllocationDenial as Denial, ExecutionAllocationDenialKind as Kind,
    ExecutionAllocationPolicy as Policy, OwnedFixedBacking,
};
use allocator_api2::vec::Vec;
use std::alloc::Layout;

/// The sole physical layout, admission, allocation, and exact-fill owner.
pub(in super::super) struct BuildingFixedBacking<T> {
    backing: OwnedFixedBacking<T>,
    element_count: usize,
    quote: u64,
    status: Option<ExecutionLeaseStatus>,
}

impl<T> BuildingFixedBacking<T> {
    pub(in super::super) fn allocate(
        element_count: usize,
        policy: Policy<'_, '_>,
    ) -> Result<Self, Denial> {
        let layout =
            Layout::array::<T>(element_count).map_err(|_| Denial::new(Kind::Layout, None))?;
        let quote = u64::try_from(layout.size()).map_err(|_| Denial::new(Kind::Layout, None))?;
        let status = match policy {
            Policy::SystemAllocation => None,
            Policy::Execution(lease) => Some(lease.status()),
        };
        if let Some(status) = &status {
            check_status(status, Some(quote))?;
        }
        let reservation = match policy {
            Policy::SystemAllocation => None,
            Policy::Execution(lease) => Some(lease.reserve_memory(quote).map_err(|error| {
                Denial::new(
                    Kind::Lease(super::super::LeaseDenial::MemoryExhausted(error)),
                    Some(quote),
                )
            })?),
        };
        if let Some(status) = &status {
            check_status(status, Some(quote))?;
        }
        // Fresh pinned allocator-api2 Global Vec: exact physical layout for
        // non-ZSTs. ZST capacity is usize::MAX; element_count still bounds writes.
        // Local values drop in reverse order: allocation before reservation.
        let mut elements = Vec::new();
        elements
            .try_reserve_exact(element_count)
            .map_err(|_| Denial::new(Kind::Allocator, Some(quote)))?;
        if std::mem::size_of::<T>() != 0 && elements.capacity() != element_count {
            return Err(Denial::new(Kind::CapacityMismatch, Some(quote)));
        }
        let building = Self {
            backing: OwnedFixedBacking::new(elements, reservation),
            element_count,
            quote,
            status,
        };
        building.check_live()?;
        Ok(building)
    }
    pub(in super::super) fn check_live(&self) -> Result<(), Denial> {
        self.status
            .as_ref()
            .map_or(Ok(()), |status| check_status(status, Some(self.quote)))
    }
    pub(in super::super) fn push(&mut self, value: T) -> Result<(), Denial> {
        self.check_live()?;
        if self.len() == self.element_count {
            return Err(self.denial(Kind::WriteBeyondReserved));
        }
        self.backing.push(value);
        self.check_live()
    }
    pub(in super::super) fn elements(&self) -> &[T] {
        self.backing.elements()
    }
    pub(in super::super) fn len(&self) -> usize {
        self.backing.elements().len()
    }
    pub(in super::super) fn element_count(&self) -> usize {
        self.element_count
    }
    pub(in super::super) fn physical_capacity(&self) -> usize {
        self.backing.capacity()
    }
    pub(in super::super) fn seal(self) -> Result<OwnedFixedBacking<T>, Denial> {
        self.check_live()?;
        if self.len() != self.element_count {
            return Err(self.denial(Kind::IncompleteSeal));
        }
        Ok(self.backing)
    }
    pub(in super::super) fn denial(&self, kind: Kind) -> Denial {
        Denial::new(kind, Some(self.quote))
    }
}

// Only the private byte author uses copying/mutable written-region access.
// Neither operation exposes the Vec, its capacity machinery, or its ticket.
impl BuildingFixedBacking<u8> {
    pub(in super::super) fn append_bytes(&mut self, bytes: &[u8]) -> Result<(), Denial> {
        self.check_live()?;
        if bytes.len() > self.element_count - self.len() {
            return Err(self.denial(Kind::WriteBeyondReserved));
        }
        self.backing.append_bytes(bytes);
        Ok(())
    }
    pub(in super::super) fn written_bytes_mut(&mut self) -> &mut [u8] {
        self.backing.written_bytes_mut()
    }
}
