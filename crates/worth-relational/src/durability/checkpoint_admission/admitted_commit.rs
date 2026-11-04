//! Operational evidence joined to an actual selected directory, never an envelope.
use super::{
    RelationalCheckpointAdmissionArtifact, RelationalCheckpointAdmissionRefusal as Refusal,
};
use crate::branch::{RelationalBranchRootSchemaAuthority, RelationalBranchVisibilityCommitment};
use crate::durability::data::RelationalCheckpointDirectory;
use crate::history::{RelationalCommitIndexEntry, RetainedCanonicalIndex};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Weak};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
mod index_predecessor;

/// Only Native's selected-directory consumer can construct this owner.
/// It cannot be converted into an original positioned canonical frame.
#[derive(Debug, Clone)]
pub(crate) struct CheckpointAdmittedCommit(Arc<AdmittedRow>);

#[derive(Debug)]
struct AdmittedRow {
    owner: Weak<super::custody::AdmissionCustody>,
    image: RelationalCheckpointAdmissionArtifact,
    index: RetainedCanonicalIndex,
    schema: Arc<RelationalBranchRootSchemaAuthority>,
    visibility: RelationalBranchVisibilityCommitment,
    _memory: Option<ExecutionMemoryReservation>,
}

impl CheckpointAdmittedCommit {
    pub(crate) fn effect_data(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<crate::durability::canonical_commit_artifact::CheckpointEffectData, Refusal> {
        let row = self
            .0
            .image
            .row(self.0.index.commit.commit_id)
            .ok_or(Refusal::InvalidProjectedMeaning)?;
        let data = crate::durability::canonical_commit_artifact::decode_checkpoint_effect(
            self.0.image.group(&row.groups[0]),
            self.0.image.group(&row.groups[6]),
            request,
        )
        .map_err(super::root_positions::projected_stop)?;
        if data.semantic.receipt() != &self.index().commit
            || data.semantic.structural().descriptor_semantics_version
                != self.0.schema.descriptor_semantics_version()
            || data.creations.iter().any(|patch| {
                patch.structural_change
                    != crate::publication::patch::data::RecordStructuralChange::Created
            })
        {
            return Err(Refusal::InvalidProjectedMeaning);
        }
        Ok(data)
    }

    pub(crate) fn require_publication_axes(
        &self,
        predecessor: &crate::branch::RelationalBranchBasisDescriptor,
        successor: &crate::branch::RelationalBranchBasisDescriptor,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        use crate::durability::authority::checkpoint_complete::admission_projection::reference::recorded_reference;
        use crate::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload;
        let row = self
            .0
            .image
            .row(self.0.index.commit.commit_id)
            .ok_or(Refusal::InvalidProjectedMeaning)?;
        let axes = BorrowedPayload(self.0.image.group(&row.groups[1]));
        let field = |name| {
            axes.field(name, request)
                .map_err(super::root_positions::projected_stop)
        };
        let prior = field("predecessor")?
            .field("reference", request)
            .map_err(super::root_positions::projected_stop)?;
        require_encoded(&recorded_reference(predecessor), prior.0, request)?;
        require_encoded(successor.branch_id(), field("branch")?.0, request)?;
        require_encoded(
            &successor.reference().generation(),
            field("generation")?.0,
            request,
        )?;
        require_encoded(&successor.truth_version(), field("truth")?.0, request)?;
        require_encoded(&successor.posture(), field("posture")?.0, request)
    }
    pub(super) fn prepare(
        directory: &Arc<RelationalCheckpointDirectory>,
        entry: &RelationalCommitIndexEntry,
        owner: Weak<super::custody::AdmissionCustody>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let image = directory
            .admission_image
            .as_ref()
            .ok_or(Refusal::MissingArtifact)?;
        let row = image
            .row(entry.commit.commit_id)
            .ok_or(Refusal::InvalidProjectedMeaning)?;
        if row.key.position != entry.position.0 {
            return Err(Refusal::InvalidProjectedMeaning);
        }
        let coordinates = Coordinates {
            receipt: &entry.commit,
            position: entry.position,
            digest: entry.payload_digest,
            roots: &entry.root,
        };
        require_encoded(&coordinates, image.group(&row.groups[3]), request)?;
        require_encoded(
            &entry.association_discovery,
            image.group(&row.groups[2]),
            request,
        )?;
        let schema: Schema = decode_fixed(image.group(&row.groups[4]), request)?;
        let visibility: Visibility = decode_fixed(image.group(&row.groups[5]), request)?;
        let authority = directory
            .source_schemas
            .get()
            .and_then(|catalog| catalog.rows.get(&schema.carrier))
            .cloned()
            .ok_or(Refusal::InvalidProjectedMeaning)?;
        if schema.version != entry.schema_version
            || authority.descriptor_semantics_version() != schema.semantics
            || authority.authority_digest() != visibility.schema
            || entry.root.schema_root() != &visibility.schema
            || entry.root.truth_root() != &visibility.storage
            || visibility.correctness != 0
        {
            return Err(Refusal::InvalidProjectedMeaning);
        }
        let index =
            RetainedCanonicalIndex::copy_original(entry, request).map_err(Refusal::Request)?;
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<AdmittedRow>())
            .map_err(|_| Refusal::CapacityOverflow)?
            .0
            .pad_to_align();
        let memory = request
            .reserve_retained_memory(layout.size() as u64)
            .map_err(|cause| Refusal::Request(cause.into()))?;
        Ok(Self(Arc::new(AdmittedRow {
            owner,
            image: image.clone(),
            index,
            schema: authority,
            visibility: RelationalBranchVisibilityCommitment::from_checkpoint(
                visibility.commitment,
            ),
            _memory: memory,
        })))
    }

    pub(crate) fn index(&self) -> &RelationalCommitIndexEntry {
        &self.0.index
    }
    pub(crate) fn schema(&self) -> &Arc<RelationalBranchRootSchemaAuthority> {
        &self.0.schema
    }
    pub(crate) fn visibility(&self) -> RelationalBranchVisibilityCommitment {
        self.0.visibility
    }
    pub(crate) fn positions(
        &self,
    ) -> Option<&crate::history::data::RequiredOutputPublicationPositions> {
        self.0.image.positions(self.0.index.commit.commit_id)
    }
    pub(crate) fn same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub(super) fn belongs_to(&self, owner: &super::custody::CheckpointAdmissionOwner) -> bool {
        owner.owns(&self.0.owner)
    }
    pub(crate) fn links_artifact(
        &self,
        artifact: &crate::history::RelationalCommitArtifact,
    ) -> bool {
        let Some(owner) = self.0.owner.upgrade() else {
            return false;
        };
        let Some((entry, source)) = artifact.envelope_residency().source_placement() else {
            return false;
        };
        entry.as_ref() == self.index()
            && owner
                .directory
                .upgrade()
                .is_some_and(|directory| Arc::ptr_eq(&directory.source, &source))
    }
}

#[derive(Serialize)]
struct Coordinates<'a> {
    receipt: &'a crate::history::data::RelationalCommitReceipt,
    position: crate::publication::patch::data::PatchStreamPosition,
    digest: [u8; 32],
    roots: &'a crate::branch::RelationalBranchRootDescriptor,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Schema {
    version: crate::schema::data::SchemaVersionId,
    semantics: crate::schema::data::DescriptorSemanticsVersion,
    carrier: [u8; 32],
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Visibility {
    storage: [u8; 32],
    schema: [u8; 32],
    correctness: u8,
    commitment: [u8; 32],
}

fn require_encoded(
    value: &impl Serialize,
    actual: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Refusal> {
    let expected = crate::durability::artifact_payload_emission::request::encode_named_charged(
        &[],
        &super::binary_encoding::BinaryFixedArrays(value),
        request,
    )
    .map_err(super::refusal::native_stop)?;
    if expected.as_ref() != actual {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    Ok(())
}
fn decode_fixed<'a, T: Deserialize<'a>>(
    bytes: &'a [u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<T, Refusal> {
    // These two closed records contain only fixed scalar fields. Refuse a
    // variable-sized impostor before Serde can traverse it without polling.
    if bytes.len() > 512 {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    crate::durability::artifact_payload_emission::request::account_work(
        request,
        bytes.len() as u64,
    )
    .map_err(super::refusal::native_stop)?;
    rmp_serde::from_slice(bytes).map_err(|_| Refusal::InvalidProjectedMeaning)
}
