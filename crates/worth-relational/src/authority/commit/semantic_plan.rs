//! The complete proposal meaning; authored commands are only one constituent.
use crate::branch::AdmittedRelationalBranchBasis;
use crate::history::data::CanonicalRetainedEntitySelection;
use crate::transactions::data::{MergedCommitPlan, TransactionId};
use crate::validation::data::InvariantPlanContract;
use std::sync::Arc;
use worth_foundational::facade::AuthoritativeRecordAspectState;

/// Prepared only from the actual retained record, never from canonical data.
#[derive(Debug, Clone)]
pub(crate) struct PreparedRetainedEntityPayload {
    // Kept for its owner-governed retention, including pending publication.
    pub(crate) _source: AdmittedRelationalBranchBasis,
    pub(crate) meaning: Arc<CanonicalRetainedEntitySelection>,
    pub(crate) payload: Option<AuthoritativeRecordAspectState>,
}

#[derive(Debug)]
pub(crate) enum PreparedRelationalCommitPlan {
    Mutation(MergedCommitPlan),
    OwnedGraphSelection {
        commands: MergedCommitPlan,
        retained: crate::mvcc::owned_graph_selection::PreparedOwnedGraphSelection,
    },
    RetainedEntitySelection {
        // Native consequences may add commands; none stand in for selection.
        commands: MergedCommitPlan,
        retained: PreparedRetainedEntityPayload,
    },
}

/// Every consumer of proposal scope must handle the selection constituent.
#[derive(Clone, Copy, Debug)]
pub(crate) enum RelationalSemanticPlan<'plan> {
    Mutation(&'plan MergedCommitPlan),
    OwnedGraphSelection {
        commands: &'plan MergedCommitPlan,
        selection: &'plan Arc<crate::history::data::CanonicalOwnedGraphSelection>,
    },
    RetainedEntitySelection {
        commands: &'plan MergedCommitPlan,
        selection: &'plan Arc<CanonicalRetainedEntitySelection>,
    },
}

impl PreparedRelationalCommitPlan {
    pub(crate) fn view(&self) -> RelationalSemanticPlan<'_> {
        match self {
            Self::Mutation(plan) => RelationalSemanticPlan::Mutation(plan),
            Self::OwnedGraphSelection { commands, retained } => {
                RelationalSemanticPlan::OwnedGraphSelection {
                    commands,
                    selection: &retained.meaning,
                }
            }
            Self::RetainedEntitySelection { commands, retained } => {
                RelationalSemanticPlan::RetainedEntitySelection {
                    commands,
                    selection: &retained.meaning,
                }
            }
        }
    }

    pub(crate) fn retained(&self) -> Option<&PreparedRetainedEntityPayload> {
        match self {
            Self::Mutation(_) | Self::OwnedGraphSelection { .. } => None,
            Self::RetainedEntitySelection { retained, .. } => Some(retained),
        }
    }
    pub(crate) fn owned_graph(
        &self,
    ) -> Option<&crate::mvcc::owned_graph_selection::PreparedOwnedGraphSelection> {
        match self {
            Self::OwnedGraphSelection { retained, .. } => Some(retained),
            _ => None,
        }
    }
}

impl<'plan> RelationalSemanticPlan<'plan> {
    pub(crate) fn with_commands<'next>(
        self,
        commands: &'next MergedCommitPlan,
    ) -> RelationalSemanticPlan<'next>
    where
        'plan: 'next,
    {
        match self {
            Self::Mutation(_) => RelationalSemanticPlan::Mutation(commands),
            Self::OwnedGraphSelection { selection, .. } => {
                RelationalSemanticPlan::OwnedGraphSelection {
                    commands,
                    selection,
                }
            }
            Self::RetainedEntitySelection { selection, .. } => {
                RelationalSemanticPlan::RetainedEntitySelection {
                    commands,
                    selection,
                }
            }
        }
    }

    /// Only the authored/consequence command constituent, not complete scope.
    pub(crate) fn commands(self) -> &'plan MergedCommitPlan {
        match self {
            Self::Mutation(plan) => plan,
            Self::RetainedEntitySelection { commands, .. }
            | Self::OwnedGraphSelection { commands, .. } => commands,
        }
    }

    pub(crate) fn transaction_id(self) -> TransactionId {
        self.commands().transaction_id
    }

    pub(crate) fn graph_selection(
        self,
    ) -> Option<&'plan crate::history::data::CanonicalOwnedGraphSelection> {
        match self {
            Self::OwnedGraphSelection { selection, .. } => Some(selection),
            _ => None,
        }
    }
    pub(crate) fn has_selection(self) -> bool {
        self.selection().is_some() || self.graph_selection().is_some()
    }
    pub(crate) fn operation_count(self) -> usize {
        self.commands().merged_intents.len() + usize::from(self.has_selection())
    }

    pub(crate) fn selection(self) -> Option<&'plan CanonicalRetainedEntitySelection> {
        match self {
            Self::Mutation(_) | Self::OwnedGraphSelection { .. } => None,
            Self::RetainedEntitySelection { selection, .. } => Some(selection),
        }
    }

    pub(crate) fn invariant_contract(self) -> InvariantPlanContract {
        let commands = self.commands().invariant_contract();
        match self {
            Self::Mutation(_) => commands,
            Self::OwnedGraphSelection { .. } => {
                commands.union(InvariantPlanContract::owned_graph_selection())
            }
            Self::RetainedEntitySelection { .. } => {
                commands.union(InvariantPlanContract::retained_entity_selection())
            }
        }
    }

    pub(crate) fn canonical_authority_kind(
        self,
    ) -> crate::history::data::CanonicalCommitAuthorityKind {
        use crate::history::data::CanonicalCommitAuthorityKind;
        match self {
            Self::Mutation(_) => CanonicalCommitAuthorityKind::VersionedTransaction,
            Self::OwnedGraphSelection { selection, .. } => {
                CanonicalCommitAuthorityKind::OwnedGraphSelection(Arc::clone(selection))
            }
            Self::RetainedEntitySelection { selection, .. } => {
                CanonicalCommitAuthorityKind::RetainedEntitySelection(Arc::clone(selection))
            }
        }
    }
}
