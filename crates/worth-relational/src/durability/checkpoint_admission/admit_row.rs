//! Admit the original immutable Branch row against the selected directory.
use super::admitted_commit::{require_encoded, CheckpointAdmittedCommit};
use super::refusal::RelationalCheckpointAdmissionRefusal as Refusal;
use super::row_custody::AdmissionCustody;
use crate::branch::root::RelationalBranchVisibilityCommitment;
use crate::durability::canonical_commit_artifact::journal_source_read::readmit_journal_commit;
use crate::durability::data::RelationalCheckpointDirectory;
use crate::history::commit::{RelationalCommitIndexEntry, RetainedCanonicalIndex};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Weak};
use worth_execution::ExecutionRequest;
use worth_foundational::CanonicalDigestId;

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

#[derive(Serialize)]
struct Coordinates<'a> {
    receipt: &'a crate::history::data::RelationalCommitReceipt,
    position: crate::publication::patch::data::PatchStreamPosition,
    digest: CanonicalDigestId,
    roots: &'a crate::branch::target::RelationalBranchRootDescriptor,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Schema {
    version: crate::schema::data::SchemaVersionId,
    semantics: crate::schema::data::DescriptorSemanticsVersion,
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
    crate::durability::artifact_payload_emission::request::account_work(
        request,
        bytes.len() as u64,
    )
    .map_err(super::refusal::native_stop)?;
    rmp_serde::from_slice(bytes).map_err(|_| Refusal::InvalidProjectedMeaning)
}
