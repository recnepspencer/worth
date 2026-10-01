use sha2::{Digest, Sha256};
use worth_proof::AdmittedBlobReleaseProof;
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimSourceBasisV1, BlobRecordV1, PersistedRecordIdentity,
    ReleaseCustodyHeadKeyV1, ReleasedGenerationReclaimBasisV1, SelectedRecordContentClass,
};

use crate::physical_runtime::{
    BlobPhysicalAllocation, PhysicalRecordReader, RecordByteLimit, RecordCountLimit,
    RecordScanOutcome, RecordScanRequest, ServingPhysicalRuntime,
};

use super::super::{scan, BlobReclaimFailure, BlobReclaimLimits};

mod inventory;
use inventory::{
    DescriptorLink, ManifestLink, ReservationLink, SelectedReleaseFact, SelectedReleaseInventory,
};
mod chain;
mod custody;
mod graph;
mod graph_types;
mod plan;
mod resources;
mod selected_descriptor;
mod transcript;
pub(in crate::physical_runtime::blob::reclaim) use custody::{
    certify as certify_pending_drop, CertifiedPendingReleasedDrop,
};
use resources::{require_grant, scratch};
pub(in crate::physical_runtime::blob::reclaim) use selected_descriptor::observe_selected_descriptor;
pub(in crate::physical_runtime) use selected_descriptor::SelectedReleasedDescriptorObservation;

/// Peak per-selected-record metadata: current facts and chain links, all
/// historical dropped IDs, plus the larger of the traversal and protection
/// stacks. The fixed decoder/read windows are charged by `memory_bytes`.
pub(in crate::physical_runtime::blob::reclaim) const SELECTION_ROSTER_ENTRY_BYTES: usize =
    std::mem::size_of::<SelectedReleaseFact>()
        + std::mem::size_of::<ManifestLink>()
        + std::mem::size_of::<DescriptorLink>()
        + std::mem::size_of::<ReservationLink>()
        + std::mem::size_of::<[u8; 16]>()
        + std::mem::size_of::<PersistedRecordIdentity>()
        + 2 * if graph::TRAVERSAL_STACK_ENTRY_BYTES > std::mem::size_of::<PersistedRecordIdentity>()
        {
            graph::TRAVERSAL_STACK_ENTRY_BYTES
        } else {
            std::mem::size_of::<PersistedRecordIdentity>()
        };

pub(in crate::physical_runtime) struct SelectedReleasedBlob {
    reader: PhysicalRecordReader,
    basis: ReleasedGenerationReclaimBasisV1,
    dropped: Vec<PersistedRecordIdentity>,
    remaining: u64,
    occupied_attempts: Vec<[u8; 16]>,
    predecessor: Option<worth_store_physical_format::ReleasedDropPredecessorV1>,
    cumulative_dropped: u64,
    terminal: bool,
    inspection: scan::ReclaimInspectionWork,
}

impl SelectedReleasedBlob {
    pub(in crate::physical_runtime) fn into_reclaim_parts(
        self,
    ) -> (
        PhysicalRecordReader,
        ReleasedGenerationReclaimBasisV1,
        Vec<PersistedRecordIdentity>,
        u64,
        Vec<[u8; 16]>,
        Vec<crate::physical_runtime::durability::PhysicalRecoveredOriginalDropNoDurableEffect>,
        Option<worth_store_physical_format::ReleasedDropPredecessorV1>,
        u64,
        bool,
    ) {
        (
            self.reader,
            self.basis,
            self.dropped,
            self.remaining,
            self.occupied_attempts,
            Vec::new(),
            self.predecessor,
            self.cumulative_dropped,
            self.terminal,
        )
    }

    pub(in crate::physical_runtime::blob::reclaim) fn is_empty(&self) -> bool {
        self.dropped.is_empty()
    }
    pub(in crate::physical_runtime::blob::reclaim) fn remaining(&self) -> u64 {
        self.remaining
    }
    pub(in crate::physical_runtime::blob::reclaim) fn inspection(
        &self,
    ) -> scan::ReclaimInspectionWork {
        self.inspection
    }
    pub(in crate::physical_runtime::blob::reclaim) fn terminal(&self) -> bool {
        self.terminal
    }
}

/// Provisional discovery yields only a claim partition. The caller must run
/// `select` again under the exact claimed protected root before any effect.
pub(in crate::physical_runtime::blob::reclaim) fn discover_release_basis(
    reader: PhysicalRecordReader,
    proof: &AdmittedBlobReleaseProof,
    limits: BlobReclaimLimits,
    allocation: &BlobPhysicalAllocation<'_>,
) -> Result<
    (
        ReleasedGenerationReclaimBasisV1,
        scan::ReclaimInspectionWork,
    ),
    BlobReclaimFailure,
> {
    require_grant(limits, allocation)?;
    let mut scratch = scratch()?;
    let mut work = scan::ReclaimInspectionWork::default();
    let mut basis = None;
    let mut selected_publication = false;
    let mut descriptors = Vec::new();
    descriptors
        .try_reserve_exact(
            usize::try_from(limits.maximum_selected_records())
                .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?,
        )
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    if descriptors.capacity() != limits.maximum_selected_records() as usize {
        return Err(BlobReclaimFailure::ScratchUnavailable);
    }
    let mut scan = reader
        .into_rebuild()
        .scan_rebuild(
            RecordScanRequest::from_start()
                .with_batch_limit(RecordCountLimit::new(1).expect("one route row"))
                .with_payload_limit(RecordByteLimit::new(8).expect("magic width")),
        )
        .map_err(BlobReclaimFailure::Scan)?;
    let mut examined = 0;
    loop {
        if examined == limits.maximum_selected_records() {
            return Err(BlobReclaimFailure::ScanBoundExhausted);
        }
        let row = match scan
            .read_next_into(&mut scratch[..8])
            .map_err(BlobReclaimFailure::Scan)?
        {
            RecordScanOutcome::Completed(_) => break,
            RecordScanOutcome::Batch(batch) => {
                let selected = batch.records().first().cloned();
                let complete = batch.is_complete();
                let payload_bytes = batch.payload(0).map_or(0, |bytes| bytes.len());
                (selected, complete, payload_bytes)
            }
        };
        let (Some(row), complete, payload_bytes) = row else {
            return Err(BlobReclaimFailure::ConflictingSelectedFate);
        };
        examined += 1;
        work.records = work
            .records
            .checked_add(1)
            .ok_or(BlobReclaimFailure::ScanBoundExhausted)?;
        scan::admit_inspected_bytes(&mut work, limits, payload_bytes)?;
        require_known_class(row.content_class())?;
        let relevant = matches!(
            row.content_class(),
            SelectedRecordContentClass::Blob(
                worth_store_physical_format::BlobRecordKind::GenerationPublished
                    | worth_store_physical_format::BlobRecordKind::DropSetManifestV3
                    | worth_store_physical_format::BlobRecordKind::ReclaimDescriptorV2
                    | worth_store_physical_format::BlobRecordKind::ReclaimDescriptorV3
            )
        );
        if relevant {
            if row.declared_payload_bytes() > scratch.len() as u64 {
                return Err(BlobReclaimFailure::InspectionWindowExhausted);
            }
            let record = PersistedRecordIdentity::new(
                row.record_id().allocation_epoch(),
                row.record_id().ordinal(),
            )
            .ok_or(BlobReclaimFailure::ConflictingSelectedFate)?;
            work.records = work
                .records
                .checked_add(1)
                .ok_or(BlobReclaimFailure::ScanBoundExhausted)?;
            let used = scan::read_selected(
                scan.protected_reader(),
                record,
                row.declared_payload_bytes(),
                &mut scratch,
                limits,
                &mut work,
            )?;
            let bytes = &scratch[..used];
            let decoded = decode_blob_record(bytes).map_err(BlobReclaimFailure::Format)?;
            if row.content_class() != SelectedRecordContentClass::Blob(decoded.kind()) {
                return Err(BlobReclaimFailure::ConflictingSelectedFate);
            }
            match decoded {
                BlobRecordV1::GenerationPublished(publication)
                    if record == proof_record(proof)? =>
                {
                    let candidate = release_basis(publication, record, bytes, proof)?;
                    merge_basis(&mut basis, candidate)?;
                    selected_publication = true;
                }
                BlobRecordV1::GenerationPublished(_) => {}
                BlobRecordV1::DropSetManifestV3(manifest) => {
                    if let BlobReclaimSourceBasisV1::ReleasedGeneration(candidate) =
                        manifest.source_basis()
                    {
                        if basis_matches_proof(candidate, proof) {
                            merge_basis(&mut basis, candidate)?;
                        }
                    }
                }
                BlobRecordV1::ReclaimDescriptorV2(descriptor)
                    if descriptor.store() == proof.store() =>
                {
                    descriptors.push(descriptor.source_basis_digest());
                }
                BlobRecordV1::ReclaimDescriptorV2(_) => {}
                BlobRecordV1::ReclaimDescriptorV3(descriptor)
                    if descriptor.base().store() == proof.store() =>
                {
                    descriptors.push(descriptor.base().source_basis_digest());
                }
                BlobRecordV1::ReclaimDescriptorV3(_) => {}
                _ => return Err(BlobReclaimFailure::ConflictingSelectedFate),
            }
        }
        if complete {
            break;
        }
    }
    let basis = basis.ok_or(BlobReclaimFailure::DeclarationMismatch)?;
    if !selected_publication
        && !descriptors
            .contains(&BlobReclaimSourceBasisV1::ReleasedGeneration(basis).digest(proof.store()))
    {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    Ok((basis, work))
}

fn release_basis(
    publication: worth_store_physical_format::BlobGenerationPublicationV1,
    record: PersistedRecordIdentity,
    bytes: &[u8],
    proof: &AdmittedBlobReleaseProof,
) -> Result<ReleasedGenerationReclaimBasisV1, BlobReclaimFailure> {
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    if publication.store() != proof.store()
        || publication.object() != proof.object()
        || publication.generation() != proof.generation()
        || digest != proof.publication_frame_sha256()
    {
        return Err(BlobReclaimFailure::DeclarationMismatch);
    }
    ReleasedGenerationReclaimBasisV1::new(
        publication,
        record,
        digest,
        proof.issuer_evidence_sha256(),
    )
    .map_err(BlobReclaimFailure::Format)
}

fn proof_record(
    proof: &AdmittedBlobReleaseProof,
) -> Result<PersistedRecordIdentity, BlobReclaimFailure> {
    PersistedRecordIdentity::new(
        proof.publication_allocation_epoch(),
        proof.publication_record_ordinal(),
    )
    .ok_or(BlobReclaimFailure::DeclarationMismatch)
}

fn basis_matches_proof(
    basis: ReleasedGenerationReclaimBasisV1,
    proof: &AdmittedBlobReleaseProof,
) -> bool {
    proof_record(proof).ok() == Some(basis.publication_record())
        && basis.publication().store() == proof.store()
        && basis.object() == proof.object()
        && basis.generation() == proof.generation()
        && basis.publication_frame_sha256() == proof.publication_frame_sha256()
        && basis.issuer_evidence_sha256() == proof.issuer_evidence_sha256()
}

fn merge_basis(
    selected: &mut Option<ReleasedGenerationReclaimBasisV1>,
    candidate: ReleasedGenerationReclaimBasisV1,
) -> Result<(), BlobReclaimFailure> {
    if selected.is_some_and(|selected| selected != candidate) {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    *selected = Some(candidate);
    Ok(())
}

fn require_known_class(class: SelectedRecordContentClass) -> Result<(), BlobReclaimFailure> {
    if class == SelectedRecordContentClass::UnknownLegacy {
        Err(BlobReclaimFailure::ConflictingSelectedFate)
    } else {
        Ok(())
    }
}

/// Definitive claimed-root selection. The caller must retain this protected
/// reader and recheck its exact root at the publication fence.
pub(in crate::physical_runtime::blob::reclaim) fn select(
    runtime: &ServingPhysicalRuntime,
    reader: PhysicalRecordReader,
    proof: &AdmittedBlobReleaseProof,
    provisional_basis: ReleasedGenerationReclaimBasisV1,
    provisional_inspection: scan::ReclaimInspectionWork,
    limits: BlobReclaimLimits,
    allocation: &BlobPhysicalAllocation<'_>,
) -> Result<SelectedReleasedBlob, BlobReclaimFailure> {
    require_grant(limits, allocation)?;
    if !basis_matches_proof(provisional_basis, proof) {
        return Err(BlobReclaimFailure::DeclarationMismatch);
    }
    let mut scratch = scratch()?;
    let mut work = scan::ReclaimInspectionWork::default();
    let (reader, mut inventory) = SelectedReleaseInventory::scan(
        reader.into_rebuild(),
        provisional_basis,
        limits,
        &mut scratch,
        &mut work,
    )?;
    let key = ReleaseCustodyHeadKeyV1::new(inventory.basis.object(), inventory.basis.generation())
        .ok_or(BlobReclaimFailure::DeclarationMismatch)?;
    let selected_head = runtime
        .selected_release_head_for_root(reader.protected_root(), key)
        .map_err(BlobReclaimFailure::ReleaseCustodySelection)?;
    let chain = chain::validate(
        &reader,
        &inventory,
        selected_head,
        limits,
        &mut scratch,
        &mut work,
    )?;
    graph::authenticate_selected_closure(
        &reader,
        &mut inventory,
        &chain,
        limits,
        &mut scratch,
        &mut work,
    )?;
    let (reader, publication_referenced, _) =
        graph::protect_external_edges(reader, &mut inventory, limits, &mut scratch, &mut work)?;
    graph::propagate_protected_subtrees(&reader, &mut inventory, limits, &mut scratch, &mut work)?;
    let planned = plan::select_post_order(
        &reader,
        &inventory,
        &chain,
        publication_referenced,
        limits,
        &mut scratch,
        &mut work,
    )?;
    Ok(SelectedReleasedBlob {
        reader,
        basis: inventory.basis,
        dropped: planned.dropped,
        remaining: planned.remaining,
        occupied_attempts: inventory.occupied_attempts,
        predecessor: chain.predecessor,
        cumulative_dropped: planned.cumulative_dropped,
        terminal: planned.terminal,
        inspection: work.checked_add(provisional_inspection)?,
    })
}
