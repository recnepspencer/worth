//! Independently admitted maintenance intent for exactly one selected manifest.
//! Its payload inventory remains routed; only the custody manifest is removed.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobAbandonmentReasonV1, BlobManifestResidueCleanup, BlobManifestResidueCleanupPhaseV1,
    BlobRecordV1, DropSetManifestV1, OriginalDropProofV1, PersistedRecordIdentity,
    BLOB_CONTROL_FRAME_MAX_BYTES,
};

use super::super::super::{context::PlanningContext, resolved_basis::ResolvedPlanningBasis};
use super::{historical_publication, selected, selected_record};

#[path = "manifest_residue/wal_fate.rs"]
pub(super) mod wal_fate;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ValidatedManifestResidueCleanup {
    intent: BlobManifestResidueCleanup,
}

impl ValidatedManifestResidueCleanup {
    pub(crate) const fn manifest_record(self) -> PersistedRecordIdentity {
        self.intent.manifest_record()
    }

    pub(crate) const fn intent(self) -> BlobManifestResidueCleanup {
        self.intent
    }

    pub(crate) const fn reserved_record(self) -> Option<PersistedRecordIdentity> {
        match self.intent.reserved_record() {
            Some(binding) => Some(binding.record()),
            None => None,
        }
    }
}

pub(super) fn verify(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    let observation_count = basis.sample.blob_manifest_residue_cleanups().len();
    let checkpoint_cutoff = context.selection.checkpoint().map(|checkpoint| {
        checkpoint
            .checkpoint()
            .compaction_cutover()
            .wal_cutoff_lsn_exclusive()
    });
    for index in 0..observation_count {
        let (range, observed, completed) = basis.sample.blob_manifest_residue_cleanups()[index];
        let intent: BlobManifestResidueCleanup = observed.into();
        let selected_generation = context
            .selection
            .root()
            .selected()
            .selector()
            .root_generation();
        if matches!(
            intent.proof(),
            OriginalDropProofV1::RecoveredNoBinding { .. }
        ) && !wal_fate::proves_recovered_no_binding(intent, basis)
        {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
        if intent.phase() == BlobManifestResidueCleanupPhaseV1::Completed {
            if completed {
                // Its visible intent is validated by the intent branch.
                continue;
            }
            // The intent may be checkpoint-covered, but a completed root
            // replacement still has independently addressed root evidence.
            if intent.store() != context.authority.media.store_identity().bytes()
                || selected_generation < intent.candidate_root_generation()
            {
                return Err(context.redo_block(basis.planning_counters(), None));
            }
            context = verify_historical_result(context, basis, intent)?;
            continue;
        }
        if intent.store() != context.authority.media.store_identity().bytes()
            || selected_generation < intent.source_root_generation()
            || (selected_generation == intent.source_root_generation() && completed)
        {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
        if selected_generation == intent.source_root_generation()
            && checkpoint_cutoff.is_none_or(|cutoff| range.start().get() < cutoff)
        {
            // A covered or unbounded interval cannot independently prove an
            // unpublished maintenance intent eligible for replay.
            return Err(context.redo_block(basis.planning_counters(), None));
        }
        if selected_generation > intent.source_root_generation() {
            // The selector may have reached the exact candidate before the
            // Completed WAL record. This branch performs no new unroute: the
            // authenticated intent and addressed candidate root are positive
            // evidence of the already-published result. Old source bytes may
            // have retired or been reused and are not a prerequisite here.
            if selected_generation < intent.candidate_root_generation() {
                return Err(context.redo_block(basis.planning_counters(), None));
            }
            context = verify_historical_result(context, basis, intent)?;
            continue;
        }
        let (next, manifest_bytes) = selected_record(
            context,
            basis,
            intent.source_root_generation(),
            intent.manifest_record(),
            BLOB_CONTROL_FRAME_MAX_BYTES as u64,
        )?;
        context = next;
        let decoded = worth_store_physical_format::decode_blob_record(&manifest_bytes);
        let (manifest, slot_generation) = match (&intent, &decoded) {
            (BlobManifestResidueCleanup::V1(_), Ok(BlobRecordV1::DropSetManifest(value))) => {
                (value, None)
            }
            (BlobManifestResidueCleanup::V2(_), Ok(BlobRecordV1::DropSetManifestV2(value))) => (
                value.drop_set(),
                Some(value.never_reserved_slot_generation()),
            ),
            _ => return Err(context.redo_block(basis.planning_counters(), None)),
        };
        if !manifest_matches_intent(intent, &manifest, &manifest_bytes) {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
        let allowed_reserved = match intent.proof() {
            OriginalDropProofV1::NeverReserved
                if slot_generation
                    .is_some_and(|generation| generation <= intent.source_root_generation()) =>
            {
                None
            }
            proof @ (OriginalDropProofV1::ProvenNoEffect { .. }
            | OriginalDropProofV1::RecoveredNoBinding { .. }) => {
                let (idempotency, fingerprint, reserved, recovered) = match proof {
                    OriginalDropProofV1::ProvenNoEffect {
                        idempotency,
                        fingerprint,
                        reserved,
                    } => (idempotency, fingerprint, reserved, false),
                    OriginalDropProofV1::RecoveredNoBinding {
                        idempotency,
                        fingerprint,
                        reserved,
                    } => (idempotency, fingerprint, Some(reserved), true),
                    OriginalDropProofV1::NeverReserved => unreachable!(),
                };
                if !recovered && !wal_fate::proves_descriptor_no_effect(intent, basis) {
                    return Err(context.redo_block(basis.planning_counters(), None));
                }
                if let Some(binding) = reserved {
                    let Some(manifest_generation) = slot_generation else {
                        return Err(context.redo_block(basis.planning_counters(), None));
                    };
                    let (next, bytes) = selected_record(
                        context,
                        basis,
                        intent.source_root_generation(),
                        binding.record(),
                        BLOB_CONTROL_FRAME_MAX_BYTES as u64,
                    )?;
                    context = next;
                    let Ok(BlobRecordV1::OriginalDropReserved(value)) =
                        worth_store_physical_format::decode_blob_record(&bytes)
                    else {
                        return Err(context.redo_block(basis.planning_counters(), None));
                    };
                    if <[u8; 32]>::from(Sha256::digest(&bytes)) != binding.frame_sha256()
                        || value.store() != intent.store()
                        || value.reclaim_attempt() != intent.reclaim_attempt()
                        || value.manifest_record() != intent.manifest_record()
                        || value.manifest_frame_sha256() != intent.manifest_frame_sha256()
                        || value.source_basis_digest() != intent.source_basis_digest()
                        || value.manifest_selected_generation() != manifest_generation
                        || value.reserved_selected_generation() > intent.source_root_generation()
                        || value.request().idempotency() != idempotency
                        || value.request().fingerprint() != fingerprint
                        || (recovered
                            && !wal_fate::recovered_reservation_key_matches(
                                intent,
                                basis.sample.policy_identity(),
                                value,
                            ))
                        || (!recovered
                            && !basis.fates.operations().iter().any(|operation| {
                                operation.identity().idempotency() == idempotency
                                    && operation.lease_issuance_generation()
                                        == value.request().lease_issuance_generation()
                                    && operation.lease_expiry_generation()
                                        == value.request().lease_expiry_generation()
                            }))
                    {
                        return Err(context.redo_block(basis.planning_counters(), None));
                    }
                    Some(binding.record())
                } else {
                    if slot_generation.is_some() {
                        return Err(context.redo_block(basis.planning_counters(), None));
                    }
                    None
                }
            }
            _ => return Err(context.redo_block(basis.planning_counters(), None)),
        };
        if matches!(intent.proof(), OriginalDropProofV1::NeverReserved)
            && !wal_fate::proves_never_reserved(intent, basis)
        {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
        let (next, declaration) = selected_record(
            context,
            basis,
            intent.source_root_generation(),
            manifest.source_basis().declaration_record(),
            156,
        )?;
        context = next;
        let (next, abandoned) = selected_record(
            context,
            basis,
            intent.source_root_generation(),
            manifest.source_basis().abandoned_record(),
            145,
        )?;
        context = next;
        let (
            Ok(BlobRecordV1::SessionDeclared(declaration_record)),
            Ok(BlobRecordV1::SessionAbandoned(abandoned_record)),
        ) = (
            worth_store_physical_format::decode_blob_record(&declaration),
            worth_store_physical_format::decode_blob_record(&abandoned),
        )
        else {
            return Err(context.redo_block(basis.planning_counters(), None));
        };
        if !custody_matches(
            intent,
            &manifest,
            declaration_record,
            &declaration,
            abandoned_record,
            &abandoned,
        ) || !expiry_matches(&context, declaration_record, abandoned_record)
        {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
        if selected_generation == intent.source_root_generation() {
            context = selected::verify_manifest_residue(
                context,
                basis,
                &manifest,
                declaration_record,
                intent.manifest_record(),
                allowed_reserved,
            )?;
            if basis.validated_manifest_cleanup.is_some() {
                return Err(context.redo_block(basis.planning_counters(), None));
            }
            basis.validated_manifest_cleanup = Some(ValidatedManifestResidueCleanup { intent });
        }
    }
    Ok(context)
}

fn manifest_matches_intent(
    intent: impl Into<BlobManifestResidueCleanup>,
    manifest: &DropSetManifestV1,
    bytes: &[u8],
) -> bool {
    let intent = intent.into();
    manifest.store() == intent.store()
        && manifest.reclaim_attempt() == intent.reclaim_attempt()
        && manifest.source_basis_digest() == intent.source_basis_digest()
        && <[u8; 32]>::from(Sha256::digest(bytes)) == intent.manifest_frame_sha256()
        && manifest
            .dropped()
            .binary_search(&intent.manifest_record())
            .is_err()
}

fn custody_matches(
    intent: BlobManifestResidueCleanup,
    manifest: &DropSetManifestV1,
    declaration: worth_store_physical_format::BlobSessionDeclarationV1,
    declaration_bytes: &[u8],
    abandoned: worth_store_physical_format::BlobSessionAbandonedV1,
    abandoned_bytes: &[u8],
) -> bool {
    let source = manifest.source_basis();
    source.session() == declaration.session()
        && source.session() == abandoned.session()
        && declaration.store() == intent.store()
        && abandoned.store() == intent.store()
        && abandoned.declaration_record() == source.declaration_record()
        && abandoned.declaration_digest() == source.declaration_frame_sha256()
        && <[u8; 32]>::from(Sha256::digest(declaration_bytes)) == source.declaration_frame_sha256()
        && <[u8; 32]>::from(Sha256::digest(abandoned_bytes)) == source.abandoned_frame_sha256()
}

fn expiry_matches(
    context: &PlanningContext,
    declaration: worth_store_physical_format::BlobSessionDeclarationV1,
    abandoned: worth_store_physical_format::BlobSessionAbandonedV1,
) -> bool {
    match abandoned.reason() {
        BlobAbandonmentReasonV1::ExplicitAbort => true,
        BlobAbandonmentReasonV1::CheckpointExpired {
            checkpoint_sequence,
        } => {
            declaration.max_checkpoint_sequence() < checkpoint_sequence.get()
                && context.selection.checkpoint().is_some_and(|checkpoint| {
                    checkpoint.checkpoint().source().identity().sequence() >= checkpoint_sequence
                })
        }
    }
}

fn verify_historical_result(
    context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    intent: BlobManifestResidueCleanup,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    let format = context.authority.record_format;
    let (context, admitted) = historical_publication::observe(
        context,
        basis,
        intent.candidate_root_generation(),
        intent.manifest_record(),
        |discovery, root, route, budget, trace, scratch| {
            if route.is_some() {
                return Ok(false);
            }
            if let Some(reserved) = intent.reserved_record() {
                if historical_publication::find_route(
                    discovery,
                    root,
                    reserved.record(),
                    format,
                    budget,
                    trace,
                    scratch,
                )?
                .is_some()
                {
                    return Ok(false);
                }
            }
            Ok(<[u8; 32]>::from(Sha256::digest(root.encode(format)))
                == intent.candidate_root_sha256()
                && root.record_count() > 0)
        },
    )?;
    if !admitted {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    Ok(context)
}

#[cfg(test)]
#[path = "manifest_residue/tests.rs"]
mod tests;
