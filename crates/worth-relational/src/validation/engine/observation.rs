use crate::runtime::PartitionEdition;
use crate::storage::overlay::{OverlayStateView, PartitionAccess, WorkingState};
use serde::{Deserialize, Serialize};

mod custom_semantics;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InvariantObservationKind {
    Committed,
    Speculative,
}

impl InvariantObservationKind {
    pub const fn diagnostic_label(self) -> &'static str {
        match self {
            Self::Committed => "committed",
            Self::Speculative => "speculative",
        }
    }
}

#[derive(Clone)]
pub(crate) struct CommittedInvariantView<'runtime> {
    committed: CommittedInvariantState<'runtime>,
    enforcement: Option<OverlayStateView<'runtime, WorkingState>>,
    enforcement_version_id: Option<crate::identity::data::VersionId>,
    proposal_identity: Option<crate::mvcc::RelationalMutationProposalIdentity>,
}

#[derive(Clone)]
enum CommittedInvariantState<'runtime> {
    Runtime(PartitionEdition),
    Branch(&'runtime crate::branch::RelationalBranchRootState),
}

impl<'runtime> CommittedInvariantView<'runtime> {
    pub(crate) fn new(state: PartitionEdition) -> Self {
        Self {
            committed: CommittedInvariantState::Runtime(state),
            enforcement: None,
            enforcement_version_id: None,
            proposal_identity: None,
        }
    }

    pub(crate) fn from_branch_with_proposed(
        state: &'runtime crate::branch::RelationalBranchRootState,
        proposed_working_state: &'runtime WorkingState,
        proposed_version_id: crate::identity::data::VersionId,
        proposal_identity: Option<crate::mvcc::RelationalMutationProposalIdentity>,
    ) -> Self {
        Self {
            committed: CommittedInvariantState::Branch(state),
            enforcement: Some(OverlayStateView::new(state, proposed_working_state)),
            enforcement_version_id: Some(proposed_version_id),
            proposal_identity,
        }
    }

    pub(crate) fn from_branch(state: &'runtime crate::branch::RelationalBranchRootState) -> Self {
        Self {
            committed: CommittedInvariantState::Branch(state),
            enforcement: None,
            enforcement_version_id: None,
            proposal_identity: None,
        }
    }

    pub(crate) fn committed_partition_access(&self) -> &dyn PartitionAccess {
        match &self.committed {
            CommittedInvariantState::Runtime(state) => state,
            CommittedInvariantState::Branch(state) => *state,
        }
    }

    pub(crate) fn enforcement_partition_access(&self) -> &dyn PartitionAccess {
        match self.enforcement.as_ref() {
            Some(state) => state,
            None => self.committed_partition_access(),
        }
    }

    pub(crate) fn before_image_partition_access(&self) -> Option<&dyn PartitionAccess> {
        self.enforcement
            .as_ref()
            .map(|_| self.committed_partition_access())
    }

    pub(crate) fn enforcement_version_id(
        &self,
        fallback: crate::identity::data::VersionId,
    ) -> crate::identity::data::VersionId {
        self.enforcement_version_id.unwrap_or(fallback)
    }

    pub(crate) fn proposal_identity(
        &self,
    ) -> Option<&crate::mvcc::RelationalMutationProposalIdentity> {
        self.proposal_identity.as_ref()
    }
}

#[derive(Clone)]
pub(crate) struct SpeculativeInvariantView<'runtime> {
    state: OverlayStateView<'runtime, WorkingState>,
    before_image_version_id: crate::identity::data::VersionId,
    proposal_identity: Option<crate::mvcc::RelationalMutationProposalIdentity>,
}

impl<'runtime> SpeculativeInvariantView<'runtime> {
    pub(crate) fn new(
        state: OverlayStateView<'runtime, WorkingState>,
        before_image_version_id: crate::identity::data::VersionId,
        proposal_identity: Option<crate::mvcc::RelationalMutationProposalIdentity>,
    ) -> Self {
        Self {
            state,
            before_image_version_id,
            proposal_identity,
        }
    }

    pub(crate) fn partition_access(&self) -> &dyn PartitionAccess {
        &self.state
    }

    pub(crate) fn before_image_partition_access(&self) -> &dyn PartitionAccess {
        self.state.base_partition_access()
    }
}

#[derive(Clone)]
pub(crate) enum InvariantObservation<'runtime> {
    Committed(CommittedInvariantView<'runtime>),
    Speculative(SpeculativeInvariantView<'runtime>),
    Probed {
        meaning: &'runtime InvariantObservation<'runtime>,
        committed: &'runtime dyn PartitionAccess,
        enforcement: &'runtime dyn PartitionAccess,
        before_image: Option<&'runtime dyn PartitionAccess>,
    },
}

impl<'runtime> InvariantObservation<'runtime> {
    pub(crate) fn with_probed_sources<'round>(
        &'round self,
        [committed, enforcement, before_image]: [&'round dyn PartitionAccess; 3],
    ) -> InvariantObservation<'round>
    where
        'runtime: 'round,
    {
        InvariantObservation::Probed {
            meaning: self,
            committed,
            enforcement,
            before_image: self.before_image_partition_access().map(|_| before_image),
        }
    }

    pub(crate) fn committed(state: PartitionEdition) -> Self {
        Self::Committed(CommittedInvariantView::new(state))
    }

    pub(crate) fn committed_branch_with_proposed(
        state: &'runtime crate::branch::RelationalBranchRootState,
        proposed_working_state: &'runtime WorkingState,
        proposed_version_id: crate::identity::data::VersionId,
        proposal_identity: Option<crate::mvcc::RelationalMutationProposalIdentity>,
    ) -> Self {
        Self::Committed(CommittedInvariantView::from_branch_with_proposed(
            state,
            proposed_working_state,
            proposed_version_id,
            proposal_identity,
        ))
    }

    pub(crate) fn committed_branch(
        state: &'runtime crate::branch::RelationalBranchRootState,
    ) -> Self {
        Self::Committed(CommittedInvariantView::from_branch(state))
    }

    pub(crate) fn speculative_with_proposal(
        state: OverlayStateView<'runtime, WorkingState>,
        before_image_version_id: crate::identity::data::VersionId,
        proposal_identity: Option<crate::mvcc::RelationalMutationProposalIdentity>,
    ) -> Self {
        Self::Speculative(SpeculativeInvariantView::new(
            state,
            before_image_version_id,
            proposal_identity,
        ))
    }

    pub(crate) fn kind(&self) -> InvariantObservationKind {
        match self {
            Self::Committed(_) => InvariantObservationKind::Committed,
            Self::Speculative(_) => InvariantObservationKind::Speculative,
            Self::Probed { meaning, .. } => meaning.kind(),
        }
    }

    pub(crate) fn committed_partition_access(&self) -> &dyn PartitionAccess {
        match self {
            Self::Committed(view) => view.committed_partition_access(),
            Self::Speculative(view) => view.partition_access(),
            Self::Probed { committed, .. } => *committed,
        }
    }

    pub(crate) fn enforcement_partition_access(&self) -> &dyn PartitionAccess {
        match self {
            Self::Committed(view) => view.enforcement_partition_access(),
            Self::Speculative(view) => view.partition_access(),
            Self::Probed { enforcement, .. } => *enforcement,
        }
    }

    pub(crate) fn enforcement_uses_committed_state(&self) -> bool {
        match self {
            Self::Committed(view) => view.enforcement.is_none(),
            Self::Speculative(_) => true,
            Self::Probed { meaning, .. } => meaning.enforcement_uses_committed_state(),
        }
    }

    pub(crate) fn before_image_uses_committed_state(&self) -> bool {
        match self {
            Self::Committed(view) => view.enforcement.is_some(),
            Self::Speculative(_) => false,
            Self::Probed { meaning, .. } => meaning.before_image_uses_committed_state(),
        }
    }

    pub(crate) fn before_image_partition_access(&self) -> Option<&dyn PartitionAccess> {
        match self {
            Self::Committed(view) => view.before_image_partition_access(),
            Self::Speculative(view) => Some(view.before_image_partition_access()),
            Self::Probed { before_image, .. } => *before_image,
        }
    }

    pub(crate) fn before_image_version_id(
        &self,
        fallback: crate::identity::data::VersionId,
    ) -> crate::identity::data::VersionId {
        match self {
            Self::Committed(_) => fallback,
            Self::Speculative(view) => view.before_image_version_id,
            Self::Probed { meaning, .. } => meaning.before_image_version_id(fallback),
        }
    }

    pub(crate) fn enforcement_version_id(
        &self,
        fallback: crate::identity::data::VersionId,
    ) -> crate::identity::data::VersionId {
        match self {
            Self::Committed(view) => view.enforcement_version_id(fallback),
            Self::Speculative(_) => fallback,
            Self::Probed { meaning, .. } => meaning.enforcement_version_id(fallback),
        }
    }

    pub(crate) fn proposal_identity(
        &self,
    ) -> Option<&crate::mvcc::RelationalMutationProposalIdentity> {
        match self {
            Self::Committed(view) => view.proposal_identity(),
            Self::Speculative(view) => view.proposal_identity.as_ref(),
            Self::Probed { meaning, .. } => meaning.proposal_identity(),
        }
    }
}
