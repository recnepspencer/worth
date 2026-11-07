//! A verified schema transition is the sole publication-time reconstruction entrance.
use super::candidate_publication::PreparedCandidateIndexPublication;
use crate::branch::RelationalBranchRoot;
use crate::indexes::data::{
    DerivedIndexMaintenanceDenial, DerivedIndexMaintenanceDenialKind as Denial,
};
use crate::runtime::IndexDefinitionReadBinding;
use crate::storage::substrate::maintenance::{CommitChange, ReconstructionLane};
use std::sync::Arc;

impl IndexDefinitionReadBinding {
    pub(super) fn prepare_schema_transition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        predecessor: &Arc<RelationalBranchRoot>,
        successor: &Arc<RelationalBranchRoot>,
        change: &Arc<CommitChange>,
    ) -> Result<Option<PreparedCandidateIndexPublication>, DerivedIndexMaintenanceDenial> {
        if !change.schema_changed()
            || predecessor.commit_id().is_none()
            || successor.commit_id() != Some(change.commit_id())
            || successor
                .commit_change()
                .is_none_or(|actual| !Arc::ptr_eq(actual, change))
            || predecessor.schema_authority().authority_digest()
                == successor.schema_authority().authority_digest()
        {
            return Err(DerivedIndexMaintenanceDenial {
                kind: Denial::CommitMismatch,
                work: Default::default(),
            });
        }
        super::basis::validate_root_predecessor(
            successor
                .canonical_envelope()
                .ok_or(DerivedIndexMaintenanceDenial {
                    kind: Denial::CommitMismatch,
                    work: Default::default(),
                })?,
            predecessor,
        )
        .map_err(|kind| DerivedIndexMaintenanceDenial {
            kind,
            work: Default::default(),
        })?;
        ReconstructionLane::enter_reconstruction(|lane| {
            lane.recover(execution, |lane, work| {
                self.prepare_index_publication(
                    execution,
                    predecessor,
                    successor,
                    change,
                    Some((lane, work)),
                )
            })
        })
    }
}
