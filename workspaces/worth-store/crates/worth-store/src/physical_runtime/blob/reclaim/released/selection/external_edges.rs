//! Selected references into a release closure from outside it. A reference
//! held by another owner protects what it names. A resume frontier of the
//! released session is that session's own residue: its edge is audited like
//! any other, and it protects nothing.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, BlobRecordV1, PersistedRecordIdentity, SelectedRecordContentClass,
};

use crate::physical_runtime::PhysicalRecordReader;

use super::super::super::{scan, BlobReclaimFailure, BlobReclaimLimits};
use super::graph_types::{own_frontier, target_session};
use super::inventory::SelectedReleaseInventory;
use super::transcript;

#[cfg(test)]
mod tests;

/// Marks incoming selected references from outside this release closure.
/// A released publication's own root edge is intentionally excluded.
pub(super) fn protect(
    reader: PhysicalRecordReader,
    inventory: &mut SelectedReleaseInventory,
    limits: BlobReclaimLimits,
    scratch: &mut [u8],
    work: &mut scan::ReclaimInspectionWork,
) -> Result<(PhysicalRecordReader, bool, [u8; 32]), BlobReclaimFailure> {
    let mut audit = ExternalEdgeAudit::new(inventory);
    let reader = scan::walk_classified(
        reader,
        limits,
        scratch,
        work,
        |record, class, _, _, bytes| {
            if class == SelectedRecordContentClass::UnknownLegacy {
                return Err(BlobReclaimFailure::ConflictingSelectedFate);
            }
            if !matches!(class, SelectedRecordContentClass::Blob(_)) {
                return Ok(());
            }
            audit.source(record, bytes)
        },
    )?;
    let (publication_referenced, digest) = audit.finish();
    Ok((reader, publication_referenced, digest))
}

/// One fold over the selected Blob records in route order. The transcript
/// binds every outside source and every edge it holds.
struct ExternalEdgeAudit<'a> {
    inventory: &'a mut SelectedReleaseInventory,
    transcript: Sha256,
    sources: u64,
    edges: u64,
    publication_referenced: bool,
}

impl<'a> ExternalEdgeAudit<'a> {
    fn new(inventory: &'a mut SelectedReleaseInventory) -> Self {
        let mut transcript = Sha256::new();
        transcript.update(transcript::EXTERNAL_DOMAIN);
        Self {
            inventory,
            transcript,
            sources: 0,
            edges: 0,
            publication_referenced: false,
        }
    }

    fn source(
        &mut self,
        record: PersistedRecordIdentity,
        bytes: &[u8],
    ) -> Result<(), BlobReclaimFailure> {
        let fact = self.inventory.fact(record).copied();
        if fact.is_some_and(|fact| fact.reachable)
            || record == self.inventory.basis.publication_record()
        {
            return Ok(());
        }
        let fact = fact.ok_or(BlobReclaimFailure::ConflictingSelectedFate)?;
        self.transcript.update([0]);
        transcript::record_id(&mut self.transcript, record);
        self.transcript.update(fact.frame_sha256);
        self.sources = self
            .sources
            .checked_add(1)
            .ok_or(BlobReclaimFailure::ScanBoundExhausted)?;
        let protects = !own_frontier(&fact, self.inventory.basis.session());
        match decode_blob_record(bytes).map_err(BlobReclaimFailure::Format)? {
            BlobRecordV1::TreeNode(node) => {
                for entry in node.entries() {
                    self.edge(record, 1, entry.record(), protects)?;
                }
            }
            BlobRecordV1::GenerationPublished(value) => {
                self.edge(record, 2, value.root_record(), protects)?;
            }
            BlobRecordV1::ChunkReuseClaim(value) => {
                self.edge(record, 3, value.source_publication(), protects)?;
                self.edge(record, 4, value.selected_chunk(), protects)?;
            }
            BlobRecordV1::ChunkReuseClaimV2(value) => {
                self.edge(record, 5, value.claim().selected_chunk(), protects)?;
            }
            BlobRecordV1::DedupeQuarantine(value) => {
                self.edge(record, 6, value.source_publication(), protects)?;
                self.edge(record, 7, value.source_chunk(), protects)?;
                self.edge(record, 8, value.conflicting_chunk(), protects)?;
            }
            BlobRecordV1::SessionFrontier(value) => {
                self.edge(record, 9, value.last_chunk_record(), protects)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn edge(
        &mut self,
        source: PersistedRecordIdentity,
        kind: u8,
        target: PersistedRecordIdentity,
        protects: bool,
    ) -> Result<(), BlobReclaimFailure> {
        let publication_record = self.inventory.basis.publication_record();
        let session = self.inventory.basis.session();
        let selected_target = self.inventory.fact(target).copied();
        let target_reachable = selected_target.is_some_and(|fact| fact.reachable);
        let same_session = selected_target.is_some_and(|fact| target_session(&fact, session));
        self.transcript.update([1, kind]);
        transcript::record_id(&mut self.transcript, source);
        transcript::record_id(&mut self.transcript, target);
        self.transcript.update([
            u8::from(target == publication_record),
            u8::from(target_reachable),
            u8::from(same_session),
        ]);
        self.edges = self
            .edges
            .checked_add(1)
            .ok_or(BlobReclaimFailure::ScanBoundExhausted)?;
        if target == publication_record {
            self.publication_referenced = true;
        }
        if protects && (target_reachable || same_session) {
            if let Some(fact) = self.inventory.fact_mut(target) {
                fact.protected = true;
            }
        }
        Ok(())
    }

    fn finish(mut self) -> (bool, [u8; 32]) {
        self.transcript.update(self.sources.to_le_bytes());
        self.transcript.update(self.edges.to_le_bytes());
        (
            self.publication_referenced,
            self.transcript.finalize().into(),
        )
    }
}
