//! Every released-generation batch: all selected typed routes are inspected
//! before recovery removes what the batch names. A batch without a
//! predecessor removes the publication; any batch may also remove resume
//! frontiers of the released session and V3-proven descendants.

use std::collections::BTreeSet;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimDescriptorV2, BlobRecordKind, BlobRecordV1,
    CurrentPhysicalRecordPlacement, DropSetManifestV3, PersistedRecordIdentity,
    ReleasedDropCustodyV1, SelectedRecordContentClass, BLOB_CHUNK_FRAME_MAX_BYTES,
};
use worth_store_recovery_physics::RecoveryOperationFate;

use super::super::super::super::{context::PlanningContext, resolved_basis::ResolvedPlanningBasis};
use super::super::super::historical_publication::{
    remaining_observation, unobserved, HistoricalFailure, HistoricalFailure::Invalid,
};
use super::super::{
    record,
    selected::{blob_store, incoming_edge},
};
use super::{
    closure_evidence::ReleasedClosureEvidence,
    external_edges::{self, ExternalBlobFact},
    graph,
    postorder::{self, TypedClosureRecord},
    route_transcript::SelectedRouteTranscript,
};

#[path = "selected/wal_admission.rs"]
mod wal_admission;

pub(super) fn verify_initial(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    descriptor: BlobReclaimDescriptorV2,
    manifest: &DropSetManifestV3,
    operation_id: [u8; 32],
    source_routes: &[CurrentPhysicalRecordPlacement],
    expected_custody: Option<ReleasedDropCustodyV1>,
    wal_descriptor: Option<PersistedRecordIdentity>,
    closure_evidence: ReleasedClosureEvidence<'_>,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    let worth_store_physical_format::BlobReclaimSourceBasisV1::ReleasedGeneration(source) =
        manifest.source_basis()
    else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    if descriptor.predecessor().is_some() != closure_evidence.has_predecessor()
        || (expected_custody.is_none()
            && (descriptor.terminal() || manifest.dropped() != [source.publication_record()]))
        || (expected_custody.is_some()
            && (manifest
                .dropped()
                .binary_search(&source.publication_record())
                .is_ok()
                != descriptor.predecessor().is_none()))
    {
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
    let pending_v3_wal = wal_admission::admits_indeterminate(
        basis,
        descriptor,
        expected_custody,
        operation_id,
        wal_descriptor,
    );
    if operations.next().is_some()
        || operation.identity().store() != descriptor.store()
        || !(matches!(
            operation.fate(),
            RecoveryOperationFate::AcknowledgedDurable
                | RecoveryOperationFate::DurableUnacknowledged
        ) || (operation.fate() == RecoveryOperationFate::Indeterminate && pending_v3_wal))
        || super::super::manifest_residue::wal_fate::expected_drop_key(
            descriptor.store(),
            descriptor.reclaim_attempt(),
            basis.sample.policy_identity(),
            operation.lease_issuance_generation(),
            operation.lease_expiry_generation(),
        ) != operation_id
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let remaining_bytes = match remaining_observation(&context, basis) {
        Ok(remaining_bytes) => remaining_bytes,
        Err(failure) => return Err(unobserved(context, basis, failure, 0)),
    };
    let Some(mut selected_routes_digest) =
        SelectedRouteTranscript::new(descriptor.source_root_generation(), source_routes.len())
    else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let format = context.authority.record_format;
    let mut discovery = context
        .authority
        .media
        .bounded_discovery(
            crate::orchestration::reader_limit::UNCOUNTED_READS,
            remaining_bytes,
        )
        .expect("nonzero selected release limits");
    let mut scratch = 0;
    // Every read of the scan fails the same way: with the limit it met, or
    // with nothing where what it read did not verify.
    let scanned = (|| -> Result<(), HistoricalFailure> {
        let mut publication_seen = false;
        let mut reservation_seen = false;
        let mut source_claims = Vec::new();
        let mut external_facts = Vec::new();
        let mut closure_facts = Vec::new();
        let mut previous_record = None;
        for placement in source_routes.iter().copied() {
            if previous_record.is_some_and(|previous| previous >= placement.record()) {
                return Err(Invalid);
            }
            previous_record = Some(placement.record());
            let class = placement.content_class();
            if class == SelectedRecordContentClass::UnknownLegacy {
                return Err(Invalid);
            }
            let SelectedRecordContentClass::Blob(kind) = class else {
                selected_routes_digest.row(placement, [0; 32]);
                continue;
            };
            let CurrentPhysicalRecordPlacement::Extent(extent) = placement else {
                return Err(Invalid);
            };
            if extent.payload_bytes() > BLOB_CHUNK_FRAME_MAX_BYTES as u64 {
                return Err(Invalid);
            }
            let bytes = record::read(
                &mut discovery,
                format,
                Some(placement),
                placement.record(),
                BLOB_CHUNK_FRAME_MAX_BYTES as u64,
                &mut basis.observed_pages.manifest_budget,
                &mut context.integrity_trace,
                &mut scratch,
            )?;
            let fact = decode_blob_record(&bytes).map_err(|_| Invalid)?;
            if fact.kind() != kind || blob_store(&fact) != descriptor.store() {
                return Err(Invalid);
            }
            selected_routes_digest.row(placement, Sha256::digest(&bytes).into());
            if expected_custody.is_some() {
                external_facts.push(ExternalBlobFact::from_record(
                    placement.record(),
                    Sha256::digest(&bytes).into(),
                    &fact,
                    source.session(),
                ));
                let tree_level = match &fact {
                    BlobRecordV1::TreeNode(node) => Some(node.occurrence().level()),
                    _ => None,
                };
                closure_facts.push(TypedClosureRecord {
                    record: placement.record(),
                    kind,
                    tree_level,
                });
            }
            if placement.record() == source.publication_record() {
                if kind != BlobRecordKind::GenerationPublished
                    || publication_seen
                    || bytes != source.publication().encode()
                {
                    return Err(Invalid);
                }
                publication_seen = true;
            } else if let BlobRecordV1::ChunkReuseClaimV2(value) = &fact {
                let claim = value.claim();
                if claim.source_publication() == source.publication_record() {
                    if value.source_publication() != source.publication()
                        || value.source_publication_frame_sha256()
                            != source.publication_frame_sha256()
                        || manifest
                            .dropped()
                            .binary_search(&claim.selected_chunk())
                            .is_ok()
                    {
                        return Err(Invalid);
                    }
                    source_claims.push(*value);
                } else if expected_custody.is_none() && incoming_edge(&fact, manifest.dropped()) {
                    return Err(Invalid);
                }
            } else if let BlobRecordV1::ChunkReuseClaim(claim) = &fact {
                if claim.source_publication() == source.publication_record() {
                    // V1 did not seal the source-publication witness. Until a
                    // selected V2 claim is admitted, removal must fail closed.
                    return Err(Invalid);
                } else if expected_custody.is_none() && incoming_edge(&fact, manifest.dropped()) {
                    return Err(Invalid);
                }
            } else if expected_custody.is_none() && incoming_edge(&fact, manifest.dropped()) {
                return Err(Invalid);
            }
            if let BlobRecordV1::GenerationPublished(value) = &fact {
                if placement.record() != source.publication_record()
                    && (value.object() == source.object() || value.session() == source.session())
                {
                    return Err(Invalid);
                }
            }
            if let BlobRecordV1::OriginalDropReserved(value) = fact {
                if value.reclaim_attempt() == descriptor.reclaim_attempt()
                    || value.manifest_record() == descriptor.manifest_record()
                {
                    if reservation_seen
                        || value.store() != descriptor.store()
                        || value.reclaim_attempt() != descriptor.reclaim_attempt()
                        || value.manifest_record() != descriptor.manifest_record()
                        || value.manifest_frame_sha256() != descriptor.manifest_frame_sha256()
                        || value.source_basis_digest() != descriptor.source_basis_digest()
                        || value.manifest_selected_generation()
                            != manifest.never_reserved_slot_generation()
                        || value.reserved_selected_generation()
                            != descriptor.source_root_generation()
                        || value.request().idempotency() != operation_id
                        || value.request().fingerprint() != operation.request_fingerprint()
                        || value.request().lease_issuance_generation()
                            != operation.lease_issuance_generation()
                        || value.request().lease_expiry_generation()
                            != operation.lease_expiry_generation()
                        || expected_custody
                            .is_some_and(|custody| value.request() != custody.request())
                    {
                        return Err(Invalid);
                    }
                    reservation_seen = true;
                }
            }
        }
        if expected_custody.is_some_and(|expected| {
            selected_routes_digest.finish() != expected.selected_route_inventory_sha256()
        }) || !(publication_seen || closure_evidence.has_predecessor())
            || !reservation_seen
        {
            return Err(Invalid);
        }
        let mut reached = BTreeSet::new();
        let closure_digest = graph::authenticate(
            &mut discovery,
            format,
            source_routes,
            source,
            closure_evidence,
            descriptor.terminal(),
            &mut basis.observed_pages.manifest_budget,
            &mut context.integrity_trace,
            &mut scratch,
            &mut reached,
        )?;
        if let Some(custody) = expected_custody {
            let audit =
                external_edges::audit(source.publication_record(), &reached, &external_facts)
                    .ok_or(Invalid)?;
            if audit.publication_referenced
                || !external_edges::validates_drop_result(
                    &reached,
                    &external_facts,
                    manifest.dropped(),
                    &audit.protected,
                    descriptor.terminal(),
                )
                || audit.digest != custody.external_edge_audit_sha256()
                || !external_edges::admits_dropped_records(
                    source.publication_record(),
                    &reached,
                    &external_facts,
                    manifest.dropped(),
                    &audit.protected,
                )
                || closure_digest != custody.authenticated_closure_edge_sha256()
                || postorder::digest(
                    source.publication_record(),
                    manifest.dropped(),
                    &closure_facts,
                ) != Some(custody.postorder_drop_sha256())
            {
                return Err(Invalid);
            }
        }
        for value in &source_claims {
            let claim = value.claim();
            if !reached.contains(&claim.selected_chunk())
                || claim.store() != source.publication().store()
                || claim.scope() != source.publication().key_scope()
                || claim.chunk_size() != source.publication().chunk_size()
            {
                return Err(Invalid);
            }
            let route = graph::routed(source_routes, claim.selected_chunk()).ok_or(Invalid)?;
            let bytes = record::read(
                &mut discovery,
                format,
                Some(route),
                claim.selected_chunk(),
                BLOB_CHUNK_FRAME_MAX_BYTES as u64,
                &mut basis.observed_pages.manifest_budget,
                &mut context.integrity_trace,
                &mut scratch,
            )?;
            if !matches!(decode_blob_record(&bytes), Ok(BlobRecordV1::Chunk(chunk))
                if chunk.occurrence().session() == source.session()
                    && chunk.occurrence().ordinal() == claim.source_ordinal()
                    && chunk.stored_digest() == claim.stored_digest()
                    && chunk.bytes().len() as u32 == claim.chunk_length()
                    && chunk.chunk_size() == claim.chunk_size())
            {
                return Err(Invalid);
            }
        }
        Ok(())
    })();
    let counters = discovery.counters();
    context.authority.media = discovery.finish();
    basis.observed_pages.historical_publication_reads = basis
        .observed_pages
        .historical_publication_reads
        .saturating_add(counters.addressed_artifacts_read);
    basis.observed_pages.historical_publication_bytes_read = basis
        .observed_pages
        .historical_publication_bytes_read
        .saturating_add(counters.bytes_read);
    basis
        .observed_pages
        .historical_publication_peak_scratch_bytes = basis
        .observed_pages
        .historical_publication_peak_scratch_bytes
        .max(scratch);
    if let Err(failure) = scanned {
        return Err(unobserved(context, basis, failure, remaining_bytes));
    }
    Ok(context)
}
