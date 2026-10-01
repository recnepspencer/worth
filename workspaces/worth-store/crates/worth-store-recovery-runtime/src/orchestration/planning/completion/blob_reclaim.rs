//! WAL v7 names a descriptor; only the selected, independently admitted
//! manifest supplies records that recovery may unroute.

#[path = "blob_reclaim/historical.rs"]
mod historical;
#[path = "blob_reclaim/legacy_basis.rs"]
mod legacy_basis;
#[path = "blob_reclaim/manifest_residue.rs"]
mod manifest_residue;
#[path = "blob_reclaim/post_verification.rs"]
mod post_verification;
#[path = "blob_reclaim/record.rs"]
pub(in crate::orchestration::planning) mod record;
#[path = "blob_reclaim/released.rs"]
mod released;
#[path = "blob_reclaim/selected.rs"]
mod selected;
#[path = "blob_reclaim/selected_release_gate.rs"]
mod selected_release_gate;
use legacy_basis::basis_valid;
pub(crate) use manifest_residue::ValidatedManifestResidueCleanup;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobAbandonmentReasonV1, BlobReclaimDescriptorV1, BlobRecordV1, BlobSessionAbandonedV1,
    BlobSessionDeclarationV1, DropSetManifestV1, PersistedPhysicalRecoveryBlobSemantic,
    PersistedRecordIdentity, BLOB_CONTROL_FRAME_MAX_BYTES,
};

use super::super::context::PlanningContext;
use super::super::resolved_basis::ResolvedPlanningBasis;
use super::historical_publication;
use record::selected_record;

pub(super) fn verify(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    for index in 0..basis.redo.projections().len() {
        let projection = &basis.redo.projections()[index];
        let operation_id = projection.operation();
        let PersistedPhysicalRecoveryBlobSemantic::RecordsDropped(binding) =
            projection.materialization().blob_semantic()
        else {
            continue;
        };
        let Some(bytes) = basis
            .redo
            .blob_semantic_record_bytes(projection.operation())
        else {
            return Err(context.redo_block(basis.planning_counters(), None));
        };
        if <[u8; 32]>::from(Sha256::digest(bytes)) != binding.record_payload_sha256() {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
        let descriptor = match worth_store_physical_format::decode_blob_record(bytes) {
            Ok(BlobRecordV1::ReclaimDescriptor(descriptor)) => descriptor,
            Ok(BlobRecordV1::ReclaimDescriptorV2(descriptor)) => {
                if descriptor.source_root_generation()
                    != projection.materialization().source_root_generation()
                    || descriptor.candidate_root_generation() != binding.candidate_root_generation()
                {
                    return Err(context.redo_block(basis.planning_counters(), None));
                }
                context = released::preflight(
                    context,
                    basis,
                    descriptor,
                    binding.record(),
                    operation_id,
                )?;
                // Only a fully verified publication-only first batch is
                // admitted here. Continuation remains fail-closed inside the
                // release verifier until selected predecessor WAL fate and
                // post-order closure are independently established.
                continue;
            }
            Ok(BlobRecordV1::ReclaimDescriptorV3(descriptor)) => {
                if descriptor.base().source_root_generation()
                    != projection.materialization().source_root_generation()
                    || descriptor.base().candidate_root_generation()
                        != binding.candidate_root_generation()
                {
                    return Err(context.redo_block(basis.planning_counters(), None));
                }
                if let Some(historical) = basis
                    .observed_pages
                    .historical_drops
                    .iter()
                    .find(|historical| historical.operation == operation_id)
                    .cloned()
                {
                    if historical.descriptor_record != binding.record()
                        || historical.descriptor != descriptor
                        || !basis
                            .redo
                            .operation_group_is_fully_materialized(operation_id)
                    {
                        return Err(context.redo_block(basis.planning_counters(), None));
                    }
                    context =
                        released::historical_redo::verify_historical(context, basis, &historical)?;
                    continue;
                }
                context = released::redo::preflight(
                    context,
                    basis,
                    descriptor,
                    binding.record(),
                    operation_id,
                )?;
                context = selected_release_gate::admit_pending_wal_release(context, basis, index)?;
                continue;
            }
            _ => return Err(context.redo_block(basis.planning_counters(), None)),
        };
        if descriptor.store() != context.authority.media.store_identity().bytes()
            || descriptor.source_root_generation()
                != projection.materialization().source_root_generation()
            || descriptor.candidate_root_generation() != binding.candidate_root_generation()
            || descriptor.manifest_record() == binding.record()
        {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
        let (next, manifest) = record::selected_record(
            context,
            basis,
            descriptor.source_root_generation(),
            descriptor.manifest_record(),
            BLOB_CONTROL_FRAME_MAX_BYTES as u64,
        )?;
        context = next;
        let decoded = worth_store_physical_format::decode_blob_record(&manifest);
        let (manifest_record, slot_generation) = match &decoded {
            Ok(BlobRecordV1::DropSetManifest(value)) => (value, None),
            Ok(BlobRecordV1::DropSetManifestV2(value)) => (
                value.drop_set(),
                Some(value.never_reserved_slot_generation()),
            ),
            _ => return Err(context.redo_block(basis.planning_counters(), None)),
        };
        if manifest_record.store() != descriptor.store()
            || manifest_record.reclaim_attempt() != descriptor.reclaim_attempt()
            || manifest_record.source_basis_digest() != descriptor.source_basis_digest()
            || manifest_record.count() != descriptor.manifest_count()
            || <[u8; 32]>::from(Sha256::digest(&manifest)) != descriptor.manifest_frame_sha256()
            || manifest_record
                .dropped()
                .binary_search(&descriptor.manifest_record())
                .is_ok()
            || manifest_record
                .dropped()
                .binary_search(&binding.record())
                .is_ok()
        {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
        let reservation = if let Some(manifest_generation) = slot_generation {
            if manifest_generation >= descriptor.source_root_generation() {
                return Err(context.redo_block(basis.planning_counters(), None));
            }
            let mut operations = basis
                .fates
                .operations()
                .iter()
                .filter(|operation| operation.identity().idempotency() == operation_id);
            let Some(operation) = operations.next() else {
                return Err(context.redo_block(basis.planning_counters(), None));
            };
            if operations.next().is_some()
                || operation.identity().store() != descriptor.store()
                || manifest_residue::wal_fate::expected_drop_key(
                    descriptor.store(),
                    descriptor.reclaim_attempt(),
                    basis.sample.policy_identity(),
                    operation.lease_issuance_generation(),
                    operation.lease_expiry_generation(),
                ) != operation_id
            {
                return Err(context.redo_block(basis.planning_counters(), None));
            }
            Some(selected::ExpectedReserved::new(
                descriptor.manifest_record(),
                descriptor.manifest_frame_sha256(),
                descriptor.store(),
                descriptor.reclaim_attempt(),
                descriptor.source_basis_digest(),
                manifest_generation,
                descriptor.source_root_generation(),
                operation_id,
                operation.request_fingerprint(),
                operation.lease_issuance_generation(),
                operation.lease_expiry_generation(),
            ))
        } else {
            None
        };
        let source = manifest_record.source_basis();
        let (next, declaration) = record::selected_record(
            context,
            basis,
            descriptor.source_root_generation(),
            source.declaration_record(),
            156,
        )?;
        context = next;
        let Ok(BlobRecordV1::SessionDeclared(declaration_record)) =
            worth_store_physical_format::decode_blob_record(&declaration)
        else {
            return Err(context.redo_block(basis.planning_counters(), None));
        };
        let (next, abandoned) = record::selected_record(
            context,
            basis,
            descriptor.source_root_generation(),
            source.abandoned_record(),
            145,
        )?;
        context = next;
        let Ok(BlobRecordV1::SessionAbandoned(abandoned_record)) =
            worth_store_physical_format::decode_blob_record(&abandoned)
        else {
            return Err(context.redo_block(basis.planning_counters(), None));
        };
        if !basis_valid(
            descriptor,
            &manifest_record,
            declaration_record,
            &declaration,
            abandoned_record,
            &abandoned,
        ) {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
        if let BlobAbandonmentReasonV1::CheckpointExpired {
            checkpoint_sequence,
        } = abandoned_record.reason()
        {
            let selected_checkpoint = context
                .selection
                .checkpoint()
                .map(|checkpoint| checkpoint.checkpoint().source().identity().sequence());
            if declaration_record.max_checkpoint_sequence() >= checkpoint_sequence.get()
                || selected_checkpoint.is_none_or(|sequence| sequence < checkpoint_sequence)
            {
                return Err(context.redo_block(basis.planning_counters(), None));
            }
        }
        let selected_generation = context
            .selection
            .root()
            .selected()
            .selector()
            .root_generation();
        if selected_generation == descriptor.source_root_generation() {
            let (next, dropped) = selected::verify_source(
                context,
                basis,
                &manifest_record,
                declaration_record,
                binding.record(),
                reservation,
            )?;
            context = next;
            basis.verified_drops.extend(dropped);
            basis.verified_drops.sort_unstable();
            if basis
                .verified_drops
                .windows(2)
                .any(|pair| pair[0] == pair[1])
            {
                return Err(context.redo_block(basis.planning_counters(), None));
            }
        } else if selected_generation >= descriptor.candidate_root_generation() {
            context = historical::verify_result(
                context,
                basis,
                descriptor,
                &manifest_record,
                binding.record(),
                binding.record_payload_sha256(),
            )?;
        } else {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
    }
    context = selected_release_gate::verify(context, basis)?;
    let context = manifest_residue::verify(context, basis)?;
    post_verification::admit(context, basis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::{BlobAbandonmentReasonV1, FailedIngestReclaimBasisV1};

    fn record(ordinal: u64) -> PersistedRecordIdentity {
        PersistedRecordIdentity::new([7; 16], ordinal).unwrap()
    }

    #[test]
    fn reclaim_basis_requires_exact_selected_declaration_and_terminal_frames() {
        let declaration = BlobSessionDeclarationV1::new(
            [1; 16],
            [2; 16],
            [3; 16],
            [4; 32],
            64 << 10,
            128 << 10,
            1 << 20,
            5,
        )
        .unwrap();
        let declaration_bytes = declaration.encode();
        let declaration_digest: [u8; 32] = Sha256::digest(&declaration_bytes).into();
        let abandoned = BlobSessionAbandonedV1::new(
            [1; 16],
            [2; 16],
            record(1),
            declaration_digest,
            BlobAbandonmentReasonV1::ExplicitAbort,
        )
        .unwrap();
        let abandoned_bytes = abandoned.encode();
        let abandoned_digest: [u8; 32] = Sha256::digest(&abandoned_bytes).into();
        let basis = FailedIngestReclaimBasisV1::new(
            [2; 16],
            record(1),
            declaration_digest,
            record(2),
            abandoned_digest,
        )
        .unwrap();
        let manifest = DropSetManifestV1::new([1; 16], [5; 16], basis, vec![record(3)]).unwrap();
        let manifest_digest: [u8; 32] = Sha256::digest(manifest.encode()).into();
        let descriptor = BlobReclaimDescriptorV1::new(
            [1; 16],
            [5; 16],
            basis.digest([1; 16]),
            record(4),
            manifest_digest,
            1,
            4,
            5,
        )
        .unwrap();
        assert!(basis_valid(
            descriptor,
            &manifest,
            declaration,
            &declaration_bytes,
            abandoned,
            &abandoned_bytes,
        ));
        let mut changed = abandoned_bytes.clone();
        changed[48] ^= 1;
        assert!(!basis_valid(
            descriptor,
            &manifest,
            declaration,
            &declaration_bytes,
            abandoned,
            &changed,
        ));
        let wrong_session = BlobSessionDeclarationV1::new(
            [1; 16],
            [9; 16],
            [3; 16],
            [4; 32],
            64 << 10,
            128 << 10,
            1 << 20,
            5,
        )
        .unwrap();
        assert!(!basis_valid(
            descriptor,
            &manifest,
            wrong_session,
            &declaration_bytes,
            abandoned,
            &abandoned_bytes,
        ));
    }
}
