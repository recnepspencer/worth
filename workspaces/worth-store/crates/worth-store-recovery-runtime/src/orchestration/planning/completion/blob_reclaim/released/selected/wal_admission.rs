//! Exact immutable-plan membership for a durable-but-unapplied V3 drop.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimDescriptorV2, BlobRecordV1, PersistedPhysicalRecoveryOperation,
    PersistedRecordIdentity, ReleasedDropCustodyV1,
};
use worth_store_recovery_physics::RecoveryOperationFate;

use super::ResolvedPlanningBasis;

pub(super) fn admits_indeterminate(
    basis: &ResolvedPlanningBasis,
    descriptor: BlobReclaimDescriptorV2,
    expected_custody: Option<ReleasedDropCustodyV1>,
    operation_id: [u8; 32],
    wal_descriptor: Option<PersistedRecordIdentity>,
) -> bool {
    // C9's Indeterminate fate is not permission to drop. Only its exact V3
    // RecordsDropped member may reach the selected-control/closure proof.
    wal_descriptor.is_some_and(|descriptor_record| {
        let mut projections = basis
            .redo
            .projections()
            .iter()
            .filter(|projection| projection.operation() == operation_id);
        let Some(projection) = projections.next() else {
            return false;
        };
        if projections.next().is_some()
            || projection.fate() != RecoveryOperationFate::Indeterminate
            || projection.materialization().source_root_generation()
                != descriptor.source_root_generation()
        {
            return false;
        }
        let PersistedPhysicalRecoveryOperation::RecordsDropped { binding, .. } =
            projection.materialization().operation()
        else {
            return false;
        };
        if binding.record() != descriptor_record
            || binding.candidate_root_generation() != descriptor.candidate_root_generation()
        {
            return false;
        }
        let Some(bytes) = basis.redo.admitted_projection_record_bytes(projection, 0) else {
            return false;
        };
        <[u8; 32]>::from(Sha256::digest(bytes)) == binding.record_payload_sha256()
            && matches!(decode_blob_record(bytes), Ok(BlobRecordV1::ReclaimDescriptorV3(value))
                if value.base() == descriptor && Some(value.custody()) == expected_custody)
    })
}
