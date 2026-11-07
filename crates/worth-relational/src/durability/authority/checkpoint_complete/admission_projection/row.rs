//! The same original-derived meanings, independent of Home chronology.
use super::{encode, outputs, reference, Denial};
use crate::durability::checkpoint_admission::{AdmissionRowKey, EncodedAdmissionRow};
use serde::{ser::SerializeSeq, Serialize, Serializer};
use worth_execution::{ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder};

pub(in crate::durability::authority::checkpoint_complete) fn encode_original(
    envelope: &crate::history::data::CanonicalCommitEnvelope,
    position: crate::publication::patch::data::PatchStreamPosition,
    payload_digest: [u8; 32],
    descriptor: &crate::branch::RelationalBranchRootDescriptor,
    carrier: [u8; 32],
    request: &ExecutionRequest<'_, '_>,
) -> Result<EncodedAdmissionRow, Denial> {
    let visibility = crate::branch::checkpoint_visibility::from_original_commit_data(
        envelope, descriptor, request,
    )
    .map_err(super::super::root_stop)?;
    let predecessor = envelope
        .branch_cell_checkpoint
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
    let correctness = match crate::branch::RelationalRootCorrectnessIndex::AuthoritativeFallback {
        crate::branch::RelationalRootCorrectnessIndex::AuthoritativeFallback => 0,
    };
    let groups = [
        encode(envelope.semantic_result(), request)?,
        encode(
            &Axes {
                predecessor: reference::predecessor(predecessor)?,
                branch: &envelope.branch_context,
                generation,
                truth,
                posture: crate::branch::RelationalBranchBasisPosture::Live,
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
                storage: *descriptor.truth_root(),
                schema: *descriptor.schema_root(),
                correctness,
                commitment: visibility.digest(),
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
    branch: &'a crate::history::data::BranchId,
    generation: worth_foundational::FoundationalBranchReferenceGeneration,
    truth: crate::branch::RelationalBranchVersion,
    posture: crate::branch::RelationalBranchBasisPosture,
}
#[derive(Serialize)]
struct Coordinates<'a> {
    receipt: &'a crate::history::data::RelationalCommitReceipt,
    position: crate::publication::patch::data::PatchStreamPosition,
    digest: [u8; 32],
    roots: &'a crate::branch::RelationalBranchRootDescriptor,
}
#[derive(Serialize)]
struct Schema {
    version: crate::schema::data::SchemaVersionId,
    semantics: crate::schema::data::DescriptorSemanticsVersion,
    carrier: [u8; 32],
}
#[derive(Serialize)]
struct Visibility {
    storage: [u8; 32],
    schema: [u8; 32],
    correctness: u8,
    commitment: [u8; 32],
}
struct Created<'a>(&'a [&'a crate::publication::patch::data::PublishedAuthoritativeRecordPatch]);
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
    patches: &'a [crate::publication::patch::data::PublishedAuthoritativeRecordPatch],
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    ExecutionArray<&'a crate::publication::patch::data::PublishedAuthoritativeRecordPatch>,
    Denial,
> {
    use crate::publication::patch::data::RecordStructuralChange::Created;
    let mut count = 0_usize;
    for patch in patches {
        crate::durability::artifact_payload_emission::request::account_work(request, 1)
            .map_err(crate::durability::checkpoint_history::capture::capture_stop)?;
        if patch.structural_change == Created {
            count += 1;
        }
    }
    let mut selected = ExecutionRequestArrayBuilder::allocate(*request, count)
        .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
    for patch in patches {
        crate::durability::artifact_payload_emission::request::account_work(request, 1)
            .map_err(crate::durability::checkpoint_history::capture::capture_stop)?;
        if patch.structural_change == Created {
            selected
                .push(patch)
                .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
        }
    }
    selected
        .seal()
        .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)
}
