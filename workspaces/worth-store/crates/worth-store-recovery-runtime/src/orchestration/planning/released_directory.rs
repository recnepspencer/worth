//! Actual selected directory reads shared by pending and completed release joins.
//! WAL metadata names the source; integrity admission supplies its exact bytes.

use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, PhysicalRecordFormatDeclaration, RecordArtifactFile,
    SelectedRecordContentClass, MAX_DERIVED_FAMILY_ROOTS,
};
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

use super::{
    manifest_entry_budget::ManifestEntryBudget, selected_source_inventory::ResidentAllowance,
};
use crate::entry::PhysicalRecoverySelectedRecordReadDenial as Denial;
use crate::integrity_ingress::RecoveryIntegrityIngressTrace;
use crate::progression::RecoverySelectedSourceInventory;

pub(super) const MAX_DIRECTORY_BYTES: u64 = (101 + MAX_DERIVED_FAMILY_ROOTS * 26) as u64;
pub(super) fn read(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    route: CurrentPhysicalRecordPlacement,
    inventory: &RecoverySelectedSourceInventory,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
    resident: &mut ResidentAllowance,
) -> Result<Vec<u8>, Denial> {
    if route.content_class() != SelectedRecordContentClass::DerivedDirectory {
        return Err(Denial::InvalidRoute);
    }
    let length = match route {
        CurrentPhysicalRecordPlacement::Extent(value) => value.payload_bytes(),
        CurrentPhysicalRecordPlacement::Inline(value) => value.payload_bytes(),
    };
    if length == 0 || length > MAX_DIRECTORY_BYTES {
        return Err(Denial::InvalidPayload);
    }
    resident
        .bytes(length)
        .map_err(|_| Denial::ResidentBoundExceeded)?;
    match route {
        CurrentPhysicalRecordPlacement::Extent(_) => {
            let mut scratch = 0;
            let (bytes, _) = super::completion::blob_reclaim::record::read_with_witness_diagnostic(
                discovery,
                format,
                Some(route),
                route.record(),
                MAX_DIRECTORY_BYTES,
                budget,
                trace,
                &mut scratch,
                resident,
            )?;
            Ok(bytes)
        }
        CurrentPhysicalRecordPlacement::Inline(placement) => {
            let selected = inventory
                .segment_pages
                .get(&(placement.segment().get(), placement.page().get()))
                .filter(|selected| {
                    selected.entry.page_cell() == placement.page_cell()
                        && selected.entry.page_generation() == placement.page_generation()
                })
                .ok_or(Denial::InvalidRoute)?;
            let page_bytes = format.page_size().bytes();
            let offset = u64::from(selected.entry.frame_index()) * u64::from(page_bytes);
            let artifact = RecordArtifactFile::Segment {
                segment: placement.segment().get(),
                generation: selected.entry.data_generation(),
            };
            budget.consume(1).map_err(|_| Denial::ManifestEntryLimit)?;
            resident.trace_slots(trace, 1).map_err(Denial::from)?;
            resident
                .transient(u64::from(page_bytes))
                .map_err(|_| Denial::ResidentBoundExceeded)?;
            let observed = discovery
                .read_segment_range(
                    placement.segment().get(),
                    selected.entry.data_generation(),
                    offset,
                    page_bytes,
                    u64::from(page_bytes),
                )
                .map_err(Denial::ManifestRead)?;
            let scope = PhysicalArtifactScope::inline_page(
                discovery.store_identity(),
                format,
                placement.page_cell(),
                PhysicalByteRange::new(offset, u64::from(page_bytes))
                    .map_err(|_| Denial::InvalidRoute)?,
            );
            let payload = crate::integrity_ingress::admit_inline_record_payload(
                &observed, scope, artifact, placement, trace,
            )
            .map_err(Denial::ManifestIntegrity)?;
            let mut bytes = Vec::new();
            bytes
                .try_reserve_exact(payload.len())
                .map_err(|cause| Denial::Allocation {
                    requested: length,
                    cause,
                })?;
            resident
                .bytes((bytes.capacity() - payload.len()) as u64)
                .map_err(|_| Denial::ResidentBoundExceeded)?;
            bytes.extend_from_slice(payload);
            Ok(bytes)
        }
    }
}
