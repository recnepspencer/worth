//! Preserve original whole canonical frames and their exact paired wire inventory.
use super::source::OriginalCanonicalCommit;
use crate::durability::{
    checkpoint_history::{
        inventory::{CanonicalCheckpointMember, CanonicalCheckpointMembers},
        publication::CanonicalCheckpointPublication,
    },
    data::*,
    medium::RelationalDurableMemberLocator,
};
use crate::history::RelationalCommitEnvelopeResidency;
use crate::publication::patch::data::PatchStreamPosition;
use std::sync::Arc;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use RelationalNativeCheckpointCaptureDenial as Denial;
pub(super) struct CanonicalOutput {
    pub(super) index: CheckpointCommitIndex,
    pub(super) inventory: CanonicalCheckpointMembers,
    pub(super) inline_bytes: usize,
    cells: ExecutionOrderedMap<PatchStreamPosition, Arc<RelationalCommitEnvelopeResidency>>,
}
impl CanonicalOutput {
    pub(super) fn prepare(
        commits: &[OriginalCanonicalCommit],
        original: &super::source::FrozenOriginalInputs,
        members: &mut ExecutionOrderedMap<
            RelationalDurableMemberLocator,
            RelationalCheckpointMember,
        >,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let prior = original.base.as_ref().map(|base| &base.directory.manifest);
        let covered = prior.map_or(0, |prior| prior.commits.len());
        let count = original.artifacts.len();
        let index = CheckpointCommitIndex::copy_for_request(
            count,
            |index| {
                if index < covered {
                    let row = &prior.expect("covered original directory").commits[index];
                    return Ok(CanonicalIndexSource {
                        receipt: &row.commit,
                        root: &row.root,
                        position: row.position,
                        digest: row.payload_digest,
                        schema_version: row.schema_version,
                        association: row.association_discovery.clone(),
                    });
                }
                let row = &commits[index - covered];
                Ok(CanonicalIndexSource {
                    receipt: &row.envelope.commit,
                    root: row.artifact.roots(),
                    position: row.position,
                    digest: row.original_payload_digest,
                    schema_version: row.envelope.schema_version,
                    association: row.envelope.association_discovery(),
                })
            },
            request,
        )
        .map_err(Denial::Request)?;
        let memory =
            CanonicalCheckpointMembers::reserve(count, request).map_err(Denial::Request)?;
        let mut rows = Vec::with_capacity(count);
        let mut cells = ExecutionOrderedMap::default();
        let mut inline_bytes = 0usize;
        if let Some(prior) = prior {
            for entry in prior.canonical_members.entries() {
                rows.push(*entry);
                members
                    .insert_cloned_native_for_request(
                        &entry.member,
                        &RelationalCheckpointMember::Carried {
                            locator: entry.member,
                        },
                        *request,
                    )
                    .map_err(map_stop)?;
            }
        }
        for artifact in original.artifacts.iter() {
            let cell = artifact.envelope_residency();
            let position = cell
                .captured_position()
                .ok_or(Denial::OriginalAuthorityUnavailable)?;
            cells
                .insert_cloned_native_for_request(&position, &cell, *request)
                .map_err(map_stop)?;
        }
        for original in commits {
            crate::execution::check_request(request).map_err(Denial::Request)?;
            let bytes = &original.original_frame;
            let member = crate::durability::log::native_checkpoint_codec::member_frame::locator(
                bytes.bytes(),
            );
            rows.push(CanonicalCheckpointMember {
                position: original.position,
                member,
            });
            if members.get(&member).is_some() {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            members
                .insert_cloned_native_for_request(
                    &member,
                    &RelationalCheckpointMember::New {
                        locator: member,
                        bytes: bytes.clone(),
                    },
                    *request,
                )
                .map_err(map_stop)?;
            inline_bytes = inline_bytes
                .checked_add(
                    crate::durability::log::native_checkpoint_codec::canonical_inline_size(
                        bytes.bytes(),
                        request,
                    )?,
                )
                .ok_or(Denial::Request(
                    crate::execution::RelationalExecutionDenialCause::MemoryOverflow,
                ))?;
        }
        Ok(Self {
            index,
            inventory: CanonicalCheckpointMembers::from_reserved(rows, memory),
            inline_bytes,
            cells,
        })
    }
    pub(super) fn publication(
        &self,
        manifest: &RelationalCheckpointManifestDraft,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CanonicalCheckpointPublication, Denial> {
        crate::durability::checkpoint_history::inventory::validate(manifest, request)
            .map_err(crate::durability::checkpoint_history::capture::capture_stop)?;
        CanonicalCheckpointPublication::capture(manifest, &self.cells, request)
    }
}
fn map_stop(cause: worth_execution::ExecutionOrderedMapDenial) -> Denial {
    crate::durability::checkpoint_history::capture::capture_stop(
        crate::durability::checkpoint_history::capture::map_stop(cause),
    )
}
