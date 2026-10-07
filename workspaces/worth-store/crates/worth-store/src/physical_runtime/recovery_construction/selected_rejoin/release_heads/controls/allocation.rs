//! Control-closure backing follows its caller's allocation owner. The V2
//! rejoin must use the carried resident ledger; pending claims retain their
//! existing separate admission until their ordinary path is cut over.

use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{DurableExtentRecordPlacement, PhysicalRecordFormatDeclaration};
use worth_store_physical_integrity::IntegrityValidatedSelectedExtentPayload;

use super::super::super::{
    control_frames::{read_extent, read_extent_with_resident, SelectedArtifactSlice},
    resident::StoreRejoinResidentLedger,
    SelectedMediaRejoinDenial as Denial,
};

pub(super) enum ControlClosureAllocation<'a> {
    Pending { maximum: u64, payloads: u64 },
    Rejoin(&'a mut StoreRejoinResidentLedger),
}

impl ControlClosureAllocation<'_> {
    pub(super) fn reserve<T>(&mut self, count: usize) -> Result<Vec<T>, Denial> {
        match self {
            Self::Rejoin(resident) => resident.reserve_vec(count).map_err(Denial::Resident),
            Self::Pending { .. } => {
                let mut values = Vec::new();
                values
                    .try_reserve_exact(count)
                    .map_err(|_| Denial::BoundExceeded)?;
                Ok(values)
            }
        }
    }

    pub(super) fn discard<T>(&mut self, values: Vec<T>) -> Result<(), Denial> {
        if let Self::Rejoin(resident) = self {
            let charged = resident.vector_bytes(&values).map_err(Denial::Resident)?;
            drop(values);
            resident.release(charged);
        }
        Ok(())
    }

    pub(super) fn read_payload(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        format: PhysicalRecordFormatDeclaration,
        placement: DurableExtentRecordPlacement,
        slices: &mut Vec<SelectedArtifactSlice>,
    ) -> Result<(Vec<u8>, IntegrityValidatedSelectedExtentPayload), Denial> {
        match self {
            Self::Rejoin(resident) => {
                read_extent_with_resident(discovery, format, placement, slices, resident)
            }
            Self::Pending { maximum, payloads } => {
                *payloads = payloads
                    .checked_add(placement.payload_bytes())
                    .filter(|bytes| *bytes <= *maximum)
                    .ok_or(Denial::BoundExceeded)?;
                read_extent(discovery, format, placement, slices)
            }
        }
    }
}
