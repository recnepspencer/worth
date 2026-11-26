//! Admit the original immutable Branch row against the selected directory.
use crate::durability::canonical_commit_artifact::journal_source_read::readmit_journal_commit;
use crate::durability::data::RelationalCheckpointDirectory;
use contracts::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal as Refusal;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Weak};
use worth_execution::ExecutionRequest;
use worth_foundational::CanonicalDigestId;
use worth_relational_branch::internal_port::branch::root::visibility::RelationalBranchVisibilityCommitment;
use worth_relational_branch::internal_port::durability::checkpoint_admission::admitted_commit::{
    require_encoded, CheckpointAdmittedCommit,
};
use worth_relational_branch::internal_port::durability::checkpoint_admission::row_custody::AdmissionCustody;
use worth_relational_branch::internal_port::history::commit::index_entry::RelationalCommitIndexEntry;
use worth_relational_branch::internal_port::history::commit::retained_index::RetainedCanonicalIndex;
use worth_relational_contracts::internal_port as contracts;

pub(super) fn admit_row(
    directory: &Arc<RelationalCheckpointDirectory>,
    entry: &RelationalCommitIndexEntry,
    owner: Weak<AdmissionCustody>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<CheckpointAdmittedCommit, Refusal> {
    let image = directory
        .admission_image
        .as_ref()
        .ok_or(Refusal::MissingArtifact)?
        .selected_image();
    let row = image
        .row(entry.commit.commit_id)
        .ok_or(Refusal::InvalidProjectedMeaning)?;
    if row.key.position != entry.position.0 {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    let coordinates = Coordinates {
        receipt: &entry.commit,
        position: entry.position,
        digest: CanonicalDigestId::new(entry.payload_digest),
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
        .and_then(|catalog| catalog.rows.get(schema.carrier.bytes()))
        .cloned()
        .ok_or(Refusal::InvalidProjectedMeaning)?;
    if schema.version != entry.schema_version
        || authority.descriptor_semantics_version() != schema.semantics
        || authority.authority_digest() != *visibility.schema.bytes()
        || entry.root.schema_root() != visibility.schema.bytes()
        || entry.root.truth_root() != visibility.storage.bytes()
        || visibility.correctness != 0
    {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    let index = RetainedCanonicalIndex::copy_original(entry, request, readmit_journal_commit)
        .map_err(Refusal::Request)?;
    CheckpointAdmittedCommit::retain(
        owner,
        image.clone(),
        index,
        authority,
        RelationalBranchVisibilityCommitment::from_checkpoint(*visibility.commitment.bytes()),
        request,
    )
}

/// Request-free on-demand row verification for settled-receipt repair paths.
pub(super) fn admit_row_without_request(
    directory: &Arc<RelationalCheckpointDirectory>,
    entry: &RelationalCommitIndexEntry,
    owner: Weak<AdmissionCustody>,
) -> Result<CheckpointAdmittedCommit, Refusal> {
    let image = directory
        .admission_image
        .as_ref()
        .ok_or(Refusal::MissingArtifact)?
        .selected_image();
    let row = image
        .row(entry.commit.commit_id)
        .ok_or(Refusal::InvalidProjectedMeaning)?;
    if row.key.position != entry.position.0 {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    let coordinates = Coordinates {
        receipt: &entry.commit,
        position: entry.position,
        digest: CanonicalDigestId::new(entry.payload_digest),
        roots: &entry.root,
    };
    require_encoded_unfunded(&coordinates, image.group(&row.groups[3]))?;
    require_encoded_unfunded(&entry.association_discovery, image.group(&row.groups[2]))?;
    let schema: Schema = decode_fixed_unfunded(image.group(&row.groups[4]))?;
    let visibility: Visibility = decode_fixed_unfunded(image.group(&row.groups[5]))?;
    let authority = directory
        .source_schemas
        .get()
        .and_then(|catalog| catalog.rows.get(schema.carrier.bytes()))
        .cloned()
        .ok_or(Refusal::InvalidProjectedMeaning)?;
    if schema.version != entry.schema_version
        || authority.descriptor_semantics_version() != schema.semantics
        || authority.authority_digest() != *visibility.schema.bytes()
        || entry.root.schema_root() != visibility.schema.bytes()
        || entry.root.truth_root() != visibility.storage.bytes()
        || visibility.correctness != 0
    {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    let index = RetainedCanonicalIndex::copy_cold(entry, readmit_journal_commit);
    Ok(CheckpointAdmittedCommit::retain_unfunded(
        owner,
        image.clone(),
        index,
        authority,
        RelationalBranchVisibilityCommitment::from_checkpoint(*visibility.commitment.bytes()),
    ))
}

#[derive(Serialize)]
struct Coordinates<'a> {
    receipt: &'a contracts::history::data::commit_receipt::RelationalCommitReceipt,
    position: contracts::publication::patch::data::patch_position::PatchStreamPosition,
    digest: CanonicalDigestId,
    roots: &'a contracts::branch::target::RelationalBranchRootDescriptor,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Schema {
    version: contracts::schema::data::SchemaVersionId,
    semantics: contracts::schema::data::continuity::descriptor_versions::DescriptorSemanticsVersion,
    carrier: CanonicalDigestId,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Visibility {
    storage: CanonicalDigestId,
    schema: CanonicalDigestId,
    correctness: u8,
    commitment: CanonicalDigestId,
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
    contracts::durability::data::recovery_stop::account_work(request, bytes.len() as u64)
        .map_err(contracts::durability::checkpoint_admission::refusal::native_stop)?;
    rmp_serde::from_slice(bytes).map_err(|_| Refusal::InvalidProjectedMeaning)
}

fn require_encoded_unfunded(value: &impl Serialize, actual: &[u8]) -> Result<(), Refusal> {
    let expected = rmp_serde::to_vec_named(
        &worth_relational_branch::internal_port::durability::checkpoint_admission::binary_encoding::BinaryFixedArrays(value),
    )
    .map_err(|_| Refusal::InvalidProjectedMeaning)?;
    if expected.as_slice() != actual {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    Ok(())
}

fn decode_fixed_unfunded<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> Result<T, Refusal> {
    if bytes.len() > 512 {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    rmp_serde::from_slice(bytes).map_err(|_| Refusal::InvalidProjectedMeaning)
}
