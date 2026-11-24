//! An owned edit consumes Full custody and freezes back into a counted cell.
use super::{Full, RelationalRegionResidency, RelationalResidencyLease};
use crate::storage::overlay::PartitionState;

pub struct ResidentFullRegionEdit {
    partition: PartitionState,
    pub(super) symbols: super::symbol_inheritance::SymbolAdmissionBasis,
}

impl RelationalResidencyLease<Full> {
    pub fn into_edit(self) -> ResidentFullRegionEdit {
        let symbols = super::symbol_inheritance::SymbolAdmissionBasis::full(&self);
        ResidentFullRegionEdit {
            symbols,
            partition: self.into_owned_partition(),
        }
    }

    pub fn has_other_observers(&self) -> bool {
        std::sync::Arc::strong_count(self.cell()) > 1
    }
}

impl ResidentFullRegionEdit {
    pub fn into_partition(self) -> PartitionState {
        self.partition
    }

    pub fn partition(&self) -> &PartitionState {
        &self.partition
    }

    pub fn partition_mut(&mut self) -> &mut PartitionState {
        &mut self.partition
    }

    pub fn freeze(self) -> RelationalResidencyLease<Full> {
        let lease = RelationalRegionResidency::resident_uncommitted(self.partition);
        self.symbols.install_full(&lease);
        lease
    }
}
