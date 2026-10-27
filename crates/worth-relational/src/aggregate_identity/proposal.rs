//! Proposal-lived initialization, retained read custody and single write consumption.
use super::changes::AggregateChanges;
use super::preparation::{PreparedAggregateArtifact, PreparedConsequences};
use super::projection::AggregateProjection;
use crate::authority::mutation::MutationEffect;
use crate::branch::{RelationalBranchRootSchemaAuthority, SelectedRelationalBranchState};
use crate::mvcc::{RelationalMaterializationTransactionMode, RelationalMutationProposalIdentity};
use crate::schema::data::AggregateIdentityContractDeclaration;
use crate::storage::overlay::WorkingState;
use crate::transactions::data::{
    CommitConflict, ConflictClass, MergedCommitPlan, TransactionCommitError,
};
use std::sync::{Arc, Mutex};

pub(crate) struct AggregateProposal {
    identity: RelationalMutationProposalIdentity,
    authority: RelationalBranchRootSchemaAuthority,
    selected: SelectedRelationalBranchState,
    // None is owner-admitted physical maintenance, not missing logical truth.
    changes: Option<AggregateChanges>,
    state: Mutex<ArtifactState>,
}

enum ArtifactState {
    Uninitialized,
    Ready(PreparedAggregateArtifact),
    Consumed(Arc<AggregateProjection>),
    Failed(TransactionCommitError),
}

#[derive(Clone, Copy)]
pub(crate) struct AggregateProposalBorrow<'a> {
    owner: &'a AggregateProposal,
    after: &'a WorkingState,
}

#[derive(Clone, Copy)]
pub(crate) struct AggregateInvocation<'a> {
    source: AggregateProposalBorrow<'a>,
    business: &'a MergedCommitPlan,
}

impl AggregateProposal {
    pub(crate) fn capture(
        authority: &RelationalBranchRootSchemaAuthority,
        selected: &SelectedRelationalBranchState,
        identity: &RelationalMutationProposalIdentity,
        effect: MutationEffect,
        materialization: Option<RelationalMaterializationTransactionMode>,
    ) -> Option<Self> {
        authority
            .registry()
            .relation_kinds
            .values()
            .any(|kind| kind.relation_integrity.aggregate_identity.is_some())
            .then(|| Self {
                identity: identity.clone(),
                authority: authority.clone(),
                selected: selected.clone(),
                changes: materialization
                    .is_none()
                    .then(|| AggregateChanges::from_applied(effect)),
                state: Mutex::new(ArtifactState::Uninitialized),
            })
    }

    pub(crate) fn borrow<'a>(&'a self, after: &'a WorkingState) -> AggregateProposalBorrow<'a> {
        AggregateProposalBorrow { owner: self, after }
    }

    fn projection(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        budget: Option<&crate::execution::RequestWorkBudget>,
        after: &WorkingState,
        business: &MergedCommitPlan,
    ) -> Result<Arc<AggregateProjection>, TransactionCommitError> {
        if business.transaction_id != self.identity.transaction_id() {
            return Err(binding_error(
                "aggregate proposal business plan affinity differs",
            ));
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match &*state {
            ArtifactState::Ready(artifact) => return Ok(Arc::clone(&artifact.projection)),
            ArtifactState::Consumed(projection) => return Ok(Arc::clone(projection)),
            ArtifactState::Failed(error) => return Err(error.clone()),
            ArtifactState::Uninitialized => {}
        }
        let changes = self
            .changes
            .as_ref()
            .ok_or_else(|| binding_error("physical maintenance reads stored logical truth"))?;
        match super::preparation::prepare(
            execution,
            budget,
            &self.authority,
            &self.selected,
            after,
            business,
            changes,
            self.identity.proposed_version_id(),
            self.identity.commit_id(),
        ) {
            Ok(artifact) => {
                let projection = Arc::clone(&artifact.projection);
                *state = ArtifactState::Ready(artifact);
                Ok(projection)
            }
            Err(error) => {
                *state = ArtifactState::Failed(error.clone());
                Err(error)
            }
        }
    }

    fn latch(&self, error: TransactionCommitError) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !matches!(*state, ArtifactState::Failed(_)) {
            *state = ArtifactState::Failed(error);
        }
    }

    fn failure(&self) -> Option<TransactionCommitError> {
        match &*self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
        {
            ArtifactState::Failed(error) => Some(error.clone()),
            _ => None,
        }
    }
}

impl<'a> AggregateProposalBorrow<'a> {
    pub(crate) fn invocation<'read>(
        self,
        business: &'read MergedCommitPlan,
    ) -> AggregateInvocation<'read>
    where
        'a: 'read,
    {
        AggregateInvocation {
            source: self,
            business,
        }
    }

    /// The only top-level initialization/consumption entry. Kernel reads never
    /// receive this request account; their invoking packet settles its report.
    pub(crate) fn take_for_commit(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        budget: Option<&crate::execution::RequestWorkBudget>,
        business: &MergedCommitPlan,
        occurrence: crate::history::data::CommitId,
    ) -> Result<Option<PreparedConsequences>, TransactionCommitError> {
        if occurrence != self.owner.identity.commit_id() {
            return Err(binding_error(
                "aggregate final occurrence differs from its proposal",
            ));
        }
        if let Some(error) = self.owner.failure() {
            return Err(error);
        }
        if self.owner.changes.is_none() {
            return Ok(None);
        }
        self.owner
            .projection(execution, budget, self.after, business)?;
        let mut state = self
            .owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match &mut *state {
            ArtifactState::Ready(artifact) => {
                let projection = Arc::clone(&artifact.projection);
                let consequences = artifact.consequences.take();
                *state = ArtifactState::Consumed(projection);
                Ok(consequences)
            }
            ArtifactState::Failed(error) => Err(error.clone()),
            _ => Err(binding_error("aggregate consequences already consumed")),
        }
    }
}

impl AggregateInvocation<'_> {
    pub(super) fn contract_count(&self, committed: bool) -> u64 {
        if committed {
            self.source
                .owner
                .selected
                .state()
                .root()
                .schema_authority()
                .registry()
                .relation_kinds
                .len() as u64
        } else {
            self.source.owner.authority.registry().relation_kinds.len() as u64
        }
    }

    pub(super) fn is_logical_mutation(&self) -> bool {
        self.source.owner.changes.is_some()
    }

    pub(super) fn contract(
        &self,
        selection: super::selection::AggregateSelection,
        committed: bool,
    ) -> Option<&AggregateIdentityContractDeclaration> {
        let authority = if committed {
            self.source.owner.selected.state().root().schema_authority()
        } else {
            &self.source.owner.authority
        };
        authority
            .registry()
            .relation_kinds
            .values()
            .filter_map(|kind| kind.relation_integrity.aggregate_identity.as_ref())
            .find(|contract| selection.matches(contract))
    }

    /// A custom planner/worker already runs under the original packet's work
    /// account. Nested Native reports charge that active frame exactly once.
    pub(super) fn projection_in_custom_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<AggregateProjection>, TransactionCommitError> {
        self.source
            .owner
            .projection(execution, None, self.source.after, self.business)
    }

    pub(super) fn latch_failure(&self, error: TransactionCommitError) {
        self.source.owner.latch(error);
    }

    pub(crate) fn settle<T>(
        &self,
        result: Result<T, TransactionCommitError>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
    ) -> Result<T, TransactionCommitError> {
        let promoted = matches!(
            &result,
            Err(TransactionCommitError::Execution {
                denial: crate::transactions::data::CommitExecutionDenial {
                    kind: crate::transactions::data::CommitExecutionDenialKind::Cause(
                        crate::execution::RelationalExecutionDenialCause::NestedStopped
                    ),
                    ..
                },
                ..
            })
        );
        if result.is_ok() || promoted {
            if let Some(error) = self.source.owner.failure() {
                control.check()?;
                crate::validation::engine::check_invariant_request(*execution)?;
                return Err(error);
            }
        }
        result
    }
}

pub(super) fn binding_error(detail: &str) -> TransactionCommitError {
    TransactionCommitError::conflict(CommitConflict::new(ConflictClass::KindSchemaMismatch {
        detail: detail.into(),
    }))
}

impl std::fmt::Debug for AggregateProposal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AggregateProposal")
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}
