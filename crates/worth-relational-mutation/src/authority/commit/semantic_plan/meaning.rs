//! Complete borrowed semantics of an admitted Mutation plan.
use crate::branch_port::history::data::canonical_commit_envelope::retained_entity_selection::CanonicalRetainedEntitySelection;
use std::sync::Arc;
use worth_relational_contracts::internal_port::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use worth_relational_contracts::internal_port::transactions::data::primitives::TransactionId;
use worth_relational_contracts::internal_port::validation::data::contracts::InvariantPlanContract;

/// Every consumer of proposal scope must handle the selection constituent.
#[derive(Clone, Copy, Debug)]
pub enum RelationalSemanticPlan<'plan> {
    Mutation(&'plan MergedCommitPlan),
    OwnedGraphSelection {
        commands: &'plan MergedCommitPlan,
        selection: &'plan Arc<crate::branch_port::history::data::canonical_commit_envelope::owned_graph_selection::CanonicalOwnedGraphSelection>,
    },
    RetainedEntitySelection {
        commands: &'plan MergedCommitPlan,
        selection: &'plan Arc<CanonicalRetainedEntitySelection>,
    },
}

impl<'plan> RelationalSemanticPlan<'plan> {
    pub fn with_commands<'next>(
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
    pub fn commands(self) -> &'plan MergedCommitPlan {
        match self {
            Self::Mutation(plan) => plan,
            Self::RetainedEntitySelection { commands, .. }
            | Self::OwnedGraphSelection { commands, .. } => commands,
        }
    }

    pub(crate) fn transaction_id(self) -> TransactionId {
        self.commands().transaction_id
    }

    pub fn graph_selection(
        self,
    ) -> Option<&'plan crate::branch_port::history::data::canonical_commit_envelope::owned_graph_selection::CanonicalOwnedGraphSelection>{
        match self {
            Self::OwnedGraphSelection { selection, .. } => Some(selection),
            Self::Mutation(_) | Self::RetainedEntitySelection { .. } => None,
        }
    }
    pub fn has_selection(self) -> bool {
        self.selection().is_some() || self.graph_selection().is_some()
    }
    pub(crate) fn operation_count(self) -> usize {
        self.commands().merged_intents.len() + usize::from(self.has_selection())
    }

    pub fn selection(self) -> Option<&'plan CanonicalRetainedEntitySelection> {
        match self {
            Self::Mutation(_) | Self::OwnedGraphSelection { .. } => None,
            Self::RetainedEntitySelection { selection, .. } => Some(selection),
        }
    }

    pub fn invariant_contract(self) -> InvariantPlanContract {
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

    pub fn canonical_authority_kind(
        self,
    ) -> crate::branch_port::history::data::canonical_commit_envelope::CanonicalCommitAuthorityKind
    {
        use crate::branch_port::history::data::canonical_commit_envelope::CanonicalCommitAuthorityKind;
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
