//! Read-only custody of the exact root reached by a commit selection.

use super::{RelationalCommitSelectionDenial, RelationalObservationReadDenial};
use crate::branch::root::RelationalBranchRoot;
use crate::execution::RelationalExecutionDenialCause;
use crate::history::{data::CommitId, retention::RelationalRetainedHistoricalRoot};
use crate::identity::data::{EntityId, RelationId};
use crate::mvcc::RelationalBranchObservation;
use crate::runtime::RelationalRuntime;
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use worth_relational_branch::internal_port::mvcc::observation::RelationalBranchObservationOwnerPort as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalPinnedCommitDenial {
    Selection(RelationalCommitSelectionDenial),
    Execution(RelationalExecutionDenialCause),
    RootUnavailable,
    Retention(crate::history::retention::RelationalRetentionAcquisitionDenial),
}

#[derive(Debug)]
pub struct RelationalPinnedCommitObservation {
    runtime_id: u64,
    root: PinnedRoot,
}

#[derive(Debug)]
enum PinnedRoot {
    Observed(RelationalBranchObservation),
    Historical(RelationalRetainedHistoricalRoot),
}

impl RelationalRuntime {
    /// Pin the exact commit reachable from this admitted branch observation.
    /// This operation retains resident history and never reconstructs a root.
    pub fn pin_commit_observation(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        observation: &RelationalBranchObservation,
        requested: CommitId,
    ) -> Result<RelationalPinnedCommitObservation, RelationalPinnedCommitDenial> {
        use RelationalPinnedCommitDenial as Denial;

        crate::execution::check_request(&execution).map_err(Denial::Execution)?;
        let selected = self.select_commit_for_pinned_read(execution, observation, requested)?;
        let observation = selected.observation();
        let root = if observation.commit_id() == Some(requested) {
            PinnedRoot::Observed(observation.clone())
        } else {
            let retained = self
                .history
                .retain_historical_root(requested)
                .map_err(Denial::Retention)?
                .filter(|root| root.root().commit_id() == Some(requested))
                .ok_or(Denial::RootUnavailable)?;
            PinnedRoot::Historical(retained)
        };
        crate::execution::check_request(&execution).map_err(Denial::Execution)?;
        Ok(RelationalPinnedCommitObservation {
            runtime_id: self.runtime_instance_id(),
            root,
        })
    }

    pub fn entity_record_at_pinned_commit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observation: &RelationalPinnedCommitObservation,
        entity: EntityId,
    ) -> Result<Option<EntityReadRecord>, RelationalObservationReadDenial> {
        observation.admit(self)?;
        self.entity_record_at_exact_root(execution, observation.root(), entity)
    }

    pub fn relation_record_at_pinned_commit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observation: &RelationalPinnedCommitObservation,
        relation: RelationId,
    ) -> Result<Option<RelationReadRecord>, RelationalObservationReadDenial> {
        observation.admit(self)?;
        self.relation_record_at_exact_root(execution, observation.root(), relation)
    }
}

impl RelationalPinnedCommitObservation {
    pub fn root_identity(&self) -> u64 {
        self.root().id()
    }

    pub fn entity_kind_declares_aspect(
        &self,
        kind: crate::identity::data::KindId,
        aspect: &worth_foundational::facade::AspectKey,
    ) -> bool {
        self.root()
            .schema_authority()
            .entity_aspect_plan(kind)
            .is_some_and(|plan| plan.contract_for(aspect).is_some())
    }

    pub fn relation_kind_declares_aspect(
        &self,
        kind: crate::identity::data::KindId,
        aspect: &worth_foundational::facade::AspectKey,
    ) -> bool {
        self.root()
            .schema_authority()
            .relation_aspect_plan(kind)
            .is_some_and(|plan| plan.contract_for(aspect).is_some())
    }

    fn admit(&self, runtime: &RelationalRuntime) -> Result<(), RelationalObservationReadDenial> {
        if self.runtime_id != runtime.runtime_instance_id() {
            return Err(RelationalObservationReadDenial::ForeignObservation);
        }
        Ok(())
    }

    fn root(&self) -> &RelationalBranchRoot {
        match &self.root {
            PinnedRoot::Observed(observation) => observation.selected_root(),
            PinnedRoot::Historical(root) => root.root(),
        }
    }
}
