//! Preserve original whole canonical frames and their exact paired wire inventory.
use super::source::OriginalCanonicalCommit;
use crate::durability::checkpoint_history::inventory::CanonicalCheckpointMember;
use crate::durability::checkpoint_history::inventory::CanonicalCheckpointMembers;
use crate::durability::checkpoint_history::publication::CanonicalCheckpointPublication;
use crate::durability::data::*;
use branch::history::commit::envelope_residency::RelationalCommitEnvelopeResidency;
use branch::history::commit::index_entry::RelationalCommitIndexEntry as Entry;
use branch::history::data::association_discovery::CanonicalCommitEnvelopeBodyOwnerPort as _;
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use std::sync::Arc;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;
use RelationalNativeCheckpointCaptureDenial as Denial;
pub(super) struct CanonicalOutput {
    /// The complete index rows; the draft pages them once roots are known.
    pub(super) rows: Vec<Entry>,
    pub(super) inventory: CanonicalCheckpointMembers,
    cells: ExecutionOrderedMap<PatchStreamPosition, Arc<RelationalCommitEnvelopeResidency>>,
}
impl CanonicalOutput {
    pub(super) fn prepare(
        commits: &[OriginalCanonicalCommit],
        artifacts: &[Arc<branch::history::commit::artifact::RelationalCommitArtifact>],
        prior: Option<&RelationalCheckpointDirectory>,
        members: &mut ExecutionOrderedMap<
            RelationalDurableMemberLocator,
            RelationalCheckpointMember,
        >,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let mut index = match prior {
            Some(prior) => {
                crate::durability::log::native_checkpoint_codec::commit_pages::prior_rows(prior)
                    .map_err(
                        contracts::durability::data::native_checkpoint_capture_denial::capture_stop,
                    )?
            }
            None => Vec::new(),
        };
        let count = artifacts.len();
        if index.len() + commits.len() != count {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        for row in commits {
            contracts::execution::request_custody::check_request(request)
                .map_err(Denial::Request)?;
            index.push(Entry {
                association_discovery: row.envelope.association_discovery(),
                commit: row.envelope.commit.clone(),
                position: row.position,
                root: row.artifact.roots().clone(),
                payload_digest: row.original_payload_digest,
                schema_version: row.envelope.schema_version,
            });
        }
        let prior = prior.map(|prior| &prior.manifest);
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
            rows: index,
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
        CanonicalCheckpointPublication::capture(&self.rows, 0, &self.cells, request)
    }
}
fn map_stop(cause: worth_execution::ExecutionOrderedMapDenial) -> Denial {
    contracts::durability::data::native_checkpoint_capture_denial::capture_stop(
        contracts::durability::data::recovery_stop::map_stop(cause),
    )
}
