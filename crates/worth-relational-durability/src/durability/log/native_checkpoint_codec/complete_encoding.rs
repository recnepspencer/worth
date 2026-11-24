//! Direct Complete output borrows the SAME sealed manifest and admitted members.
use crate::durability::data::CapturedCheckpointMembers;
use crate::durability::data::NativeCheckpointSectionBytes;
use crate::durability::data::RelationalCheckpointManifest;
use contracts::durability::artifact_payload_emission::request::{encode_named, named_size};
use contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;
use serde::{
    ser::{SerializeSeq, SerializeStruct},
    Serialize,
};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_relational_contracts::internal_port as contracts;

struct InlineMembers<'a> {
    members: &'a CapturedCheckpointMembers,
    embedded: bool,
}
impl Serialize for InlineMembers<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let entries = self.members.entries();
        let mut sequence =
            serializer.serialize_seq(Some(if self.embedded { entries.len() } else { 0 }))?;
        if self.embedded {
            for member in entries {
                let worth_relational_storage::internal_port::storage::data::checkpoint_member::RelationalCheckpointMember::New { locator, bytes } =
                    member
                else {
                    return Err(serde::ser::Error::custom(
                        "Complete inline output lacks original member bytes",
                    ));
                };
                sequence.serialize_element(&(
                    locator,
                    super::checkpoint_wire::ByteString(bytes.bytes()),
                ))?;
            }
        }
        sequence.end()
    }
}
struct CompleteWire<'a> {
    manifest: &'a RelationalCheckpointManifest,
    members: InlineMembers<'a>,
}
impl Serialize for CompleteWire<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("CheckpointWire", 3)?;
        state.serialize_field("manifest", self.manifest)?;
        state.serialize_field("members", &self.members)?;
        state.serialize_field("envelopes", &[] as &[u8])?;
        state.end()
    }
}
pub(crate) fn encode_complete(
    manifest: &RelationalCheckpointManifest,
    members: &CapturedCheckpointMembers,
    execution: &ExecutionRequest<'_, '_>,
    embedded: bool,
    canonical_inline_bytes: usize,
) -> Result<(ExecutionImmutableBytes, NativeCheckpointSectionBytes), Denial> {
    let wire = CompleteWire {
        manifest,
        members: InlineMembers { members, embedded },
    };
    let bytes = encode_named(super::CHECKPOINT_MAGIC, &wire, execution).map_err(stop)?;
    let total = bytes.len();
    let branch_roots = named_size(&manifest.roots, execution).map_err(stop)?;
    let branch_cells = named_size(&manifest.branch_cells, execution).map_err(stop)?;
    let partition_mirror = named_size(&manifest.mirror, execution).map_err(stop)?;
    let derived_indexes = named_size(&manifest.index_definitions, execution).map_err(stop)?;
    let envelopes = if embedded { canonical_inline_bytes } else { 0 };
    let classified = branch_roots
        .checked_add(branch_cells)
        .and_then(|n| n.checked_add(partition_mirror))
        .and_then(|n| n.checked_add(derived_indexes))
        .and_then(|n| n.checked_add(envelopes))
        .ok_or(Denial::Request(
            contracts::execution::denial_cause::RelationalExecutionDenialCause::MemoryOverflow,
        ))?;
    let framing_and_metadata = total
        .checked_sub(classified)
        .ok_or(Denial::OriginalAuthorityUnavailable)?;
    Ok((
        bytes,
        NativeCheckpointSectionBytes {
            total,
            envelopes,
            branch_roots,
            branch_cells,
            partition_mirror,
            derived_indexes,
            framing_and_metadata,
        },
    ))
}
fn stop(cause: contracts::durability::data::recovery_stop::RelationalRecoveryStop) -> Denial {
    contracts::durability::data::native_checkpoint_capture_denial::capture_stop(cause)
}

pub(crate) fn canonical_inline_size(
    bytes: &[u8],
    execution: &ExecutionRequest<'_, '_>,
) -> Result<usize, Denial> {
    named_size(&super::checkpoint_wire::ByteString(bytes), execution).map_err(stop)
}
