//! Preserve original whole canonical frames and their exact paired wire inventory.
use super::source::OriginalCanonicalCommit;
use crate::durability::checkpoint_history::inventory::CanonicalCheckpointMember;
use crate::durability::checkpoint_history::inventory::CanonicalCheckpointMembers;
use crate::durability::checkpoint_history::publication::CanonicalCheckpointPublication;
use crate::durability::data::*;
use branch::history::commit::envelope_residency::RelationalCommitEnvelopeResidency;
use branch::history::data::association_discovery::CanonicalCommitEnvelopeBodyOwnerPort as _;
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use std::sync::Arc;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;
use RelationalNativeCheckpointCaptureDenial as Denial;
pub(super) struct CanonicalOutput {
    pub(super) index: CheckpointCommitIndex,
    pub(super) inventory: CanonicalCheckpointMembers,
    cells: ExecutionOrderedMap<PatchStreamPosition, Arc<RelationalCommitEnvelopeResidency>>,
}
impl CanonicalOutput {
    pub(super) fn prepare(
        commits: &[OriginalCanonicalCommit],
        artifacts: &[Arc<branch::history::commit::artifact::RelationalCommitArtifact>],
        prior: Option<&RelationalCheckpointManifest>,
        members: &mut ExecutionOrderedMap<
            RelationalDurableMemberLocator,
            RelationalCheckpointMember,
        >,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let covered = prior.map_or(0, |prior| prior.commits.len());
        let count = artifacts.len();
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
        for artifact in artifacts {
            let cell = artifact.envelope_residency();
            let position = cell
                .captured_position()
                .ok_or(Denial::OriginalAuthorityUnavailable)?;
            cells
                .insert_cloned_native_for_request(&position, &cell, *request)
                .map_err(map_stop)?;
        }
        for original in commits {
            contracts::execution::request_custody::check_request(request)
                .map_err(Denial::Request)?;
            let bytes = &original.original_frame;
            let member =
                worth_relational_storage::internal_port::storage::data::member_frame::locator(
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
        }
        Ok(Self {
            index,
            inventory: CanonicalCheckpointMembers::from_reserved(rows, memory),
            cells,
        })
    }
    pub(super) fn publication(
        &self,
        manifest: &RelationalCheckpointManifestDraft,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CanonicalCheckpointPublication, Denial> {
        crate::durability::checkpoint_history::inventory::validate(manifest, request)
            .map_err(contracts::durability::data::native_checkpoint_capture_denial::capture_stop)?;
        CanonicalCheckpointPublication::capture(manifest, &self.cells, request)
    }
}
fn map_stop(cause: worth_execution::ExecutionOrderedMapDenial) -> Denial {
    contracts::durability::data::native_checkpoint_capture_denial::capture_stop(
        contracts::durability::data::recovery_stop::map_stop(cause),
    )
}
