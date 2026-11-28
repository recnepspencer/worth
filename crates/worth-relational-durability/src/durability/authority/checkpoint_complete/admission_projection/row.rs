//! The same original-derived meanings, independent of Home chronology.
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyOwnerPort as _;
use super::encode;
use super::outputs;
use super::reference;
use crate::durability::checkpoint_admission::EncodedAdmissionRow;
use contracts::branch::version::RelationalBranchVersionOwnerPort as _;
use contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;
use serde::{ser::SerializeSeq, Serialize, Serializer};
use worth_execution::{ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::CanonicalDigestId;
use worth_relational_branch::internal_port::durability::checkpoint_admission::read_image::AdmissionRowKey;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use worth_relational_contracts::internal_port as contracts;

pub(in crate::internal_port::durability::authority::checkpoint_complete) fn encode_original(
    envelope: &worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope,
    position: contracts::publication::patch::data::patch_position::PatchStreamPosition,
    payload_digest: CanonicalDigestId,
    descriptor: &contracts::branch::target::RelationalBranchRootDescriptor,
    carrier: CanonicalDigestId,
    request: &ExecutionRequest<'_, '_>,
) -> Result<EncodedAdmissionRow, Denial> {
    let visibility = worth_relational_branch::internal_port::branch::checkpoint_visibility::from_original_commit_data(
        envelope, descriptor, request,
    )
    .map_err(super::super::root_stop)?;
    let predecessor = (*envelope.owner_branch_cell_checkpoint())
        .as_ref()
        .ok_or(Denial::OriginalAuthorityUnavailable)?;
    let generation = predecessor
        .observation
        .generation()
        .checked_advance()
        .map_err(|_| Denial::OriginalAuthorityUnavailable)?;
    let truth = predecessor
        .truth_version
        .checked_advance()
        .ok_or(Denial::OriginalAuthorityUnavailable)?;
    let created = created_patches(&envelope.patch.authoritative_record_patches, request)?;
    let correctness =
        match worth_relational_branch::internal_port::branch::root::RelationalRootCorrectnessIndex::AuthoritativeFallback {
            worth_relational_branch::internal_port::branch::root::RelationalRootCorrectnessIndex::AuthoritativeFallback => 0,
        };
    let groups = [
        encode(envelope.semantic_result(), request)?,
        encode(
            &Axes {
                predecessor: reference::predecessor(predecessor)?,
                branch: &envelope.branch_context,
                generation,
                truth,
                posture: worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisPosture::Live,
            },
            request,
        )?,
        encode(&envelope.association_discovery(), request)?,
        encode(
            &Coordinates {
                receipt: &envelope.commit,
                position,
                digest: payload_digest,
                roots: descriptor,
            },
            request,
        )?,
        encode(
            &Schema {
                version: envelope.schema_version,
                semantics: envelope.descriptor_semantics_version,
                carrier,
            },
            request,
        )?,
        encode(
            &Visibility {
                storage: CanonicalDigestId::new(*descriptor.truth_root()),
                schema: CanonicalDigestId::new(*descriptor.schema_root()),
                correctness,
                commitment: CanonicalDigestId::new(visibility.digest()),
            },
            request,
        )?,
        encode(&Created(&created), request)?,
        encode(&outputs::Projection(envelope), request)?,
    ];
    Ok(EncodedAdmissionRow {
        key: AdmissionRowKey {
            position: position.0,
            commit: envelope.commit.commit_id.0,
        },
        groups,
    })
}

#[derive(Serialize)]
struct Axes<'a> {
    predecessor: reference::Predecessor<'a>,
    branch: &'a contracts::history::data::coordinates::BranchId,
    generation: worth_foundational::FoundationalBranchReferenceGeneration,
    truth: contracts::branch::version::RelationalBranchVersion,
    posture: worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisPosture,
}
#[derive(Serialize)]
struct Coordinates<'a> {
    receipt: &'a contracts::history::data::commit_receipt::RelationalCommitReceipt,
    position: contracts::publication::patch::data::patch_position::PatchStreamPosition,
    digest: CanonicalDigestId,
    roots: &'a contracts::branch::target::RelationalBranchRootDescriptor,
}
#[derive(Serialize)]
struct Schema {
    version: contracts::schema::data::SchemaVersionId,
    semantics: contracts::schema::data::continuity::descriptor_versions::DescriptorSemanticsVersion,
    carrier: CanonicalDigestId,
}
#[derive(Serialize)]
struct Visibility {
    storage: CanonicalDigestId,
    schema: CanonicalDigestId,
    correctness: u8,
    commitment: CanonicalDigestId,
}
struct Created<'a>(&'a [&'a contracts::publication::patch::data::authoritative_patch_envelope::PublishedAuthoritativeRecordPatch]);
impl Serialize for Created<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for patch in self.0 {
            sequence.serialize_element(patch)?;
        }
        sequence.end()
    }
}

fn created_patches<'a>(
    patches: &'a [contracts::publication::patch::data::authoritative_patch_envelope::PublishedAuthoritativeRecordPatch],
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    ExecutionArray<&'a contracts::publication::patch::data::authoritative_patch_envelope::PublishedAuthoritativeRecordPatch>,
    Denial,
>{
    use contracts::publication::patch::data::record_structural_change::RecordStructuralChange::Created;
    let mut count = 0_usize;
    for patch in patches {
        contracts::durability::data::recovery_stop::account_work(request, 1)
            .map_err(contracts::durability::data::native_checkpoint_capture_denial::capture_stop)?;
        if patch.structural_change == Created {
            count += 1;
        }
    }
    let mut selected = ExecutionRequestArrayBuilder::allocate(*request, count).map_err(
        contracts::durability::data::native_checkpoint_capture_denial::array_capture_stop,
    )?;
    for patch in patches {
        contracts::durability::data::recovery_stop::account_work(request, 1)
            .map_err(contracts::durability::data::native_checkpoint_capture_denial::capture_stop)?;
        if patch.structural_change == Created {
            selected.push(patch).map_err(
                contracts::durability::data::native_checkpoint_capture_denial::array_capture_stop,
            )?;
        }
    }
    selected
        .seal()
        .map_err(contracts::durability::data::native_checkpoint_capture_denial::array_capture_stop)
}
