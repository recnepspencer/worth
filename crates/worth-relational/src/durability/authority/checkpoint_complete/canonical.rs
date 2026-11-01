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
        members: &mut ExecutionOrderedMap<
            RelationalDurableMemberLocator,
            RelationalCheckpointMember,
        >,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let index = CheckpointCommitIndex::copy_for_request(
            commits.len(),
            |index| {
                let row = &commits[index];
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
            CanonicalCheckpointMembers::reserve(commits.len(), request).map_err(Denial::Request)?;
        let mut rows = Vec::with_capacity(commits.len());
        let mut cells = ExecutionOrderedMap::default();
        let mut inline_bytes = 0usize;
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
            cells
                .insert_cloned_native_for_request(
                    &original.position,
                    &original.artifact.envelope_residency(),
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
