//! Own the normalized inputs carried through commit preparation.
pub use mutation::authority::commit::semantic_plan::meaning;
pub(crate) use mutation::authority::commit::semantic_plan::meaning::RelationalSemanticPlan;
pub use mutation::authority::commit::semantic_plan::retained_entity_payload::PreparedRetainedEntityPayload;
use worth_relational_mutation::internal_port as mutation;

#[derive(Debug)]
pub enum PreparedRelationalCommitPlan {
    Mutation(mutation::mvcc::transaction::normalized_plan::OwnedNormalizedPlan),
    OwnedGraphSelection {
        commands: mutation::mvcc::transaction::normalized_plan::OwnedNormalizedPlan,
        retained: mutation::mvcc::owned_graph_selection::prepared::PreparedOwnedGraphSelection,
    },
    RetainedEntitySelection {
        // Native consequences may add commands; none stand in for selection.
        commands: mutation::mvcc::transaction::normalized_plan::OwnedNormalizedPlan,
        retained: PreparedRetainedEntityPayload,
    },
}

impl PreparedRelationalCommitPlan {
    pub fn view(&self) -> RelationalSemanticPlan<'_> {
        match self {
            Self::Mutation(plan) => RelationalSemanticPlan::Mutation(plan.plan()),
            Self::OwnedGraphSelection { commands, retained } => {
                RelationalSemanticPlan::OwnedGraphSelection {
                    commands: commands.plan(),
                    selection: &retained.meaning,
                }
            }
            Self::RetainedEntitySelection { commands, retained } => {
                RelationalSemanticPlan::RetainedEntitySelection {
                    commands: commands.plan(),
                    selection: &retained.meaning,
                }
            }
        }
    }

    pub fn retained(&self) -> Option<&PreparedRetainedEntityPayload> {
        match self {
            Self::Mutation(_) | Self::OwnedGraphSelection { .. } => None,
            Self::RetainedEntitySelection { retained, .. } => Some(retained),
        }
    }
    pub fn owned_graph(
        &self,
    ) -> Option<&mutation::mvcc::owned_graph_selection::prepared::PreparedOwnedGraphSelection> {
        match self {
            Self::OwnedGraphSelection { retained, .. } => Some(retained),
            Self::Mutation(_) | Self::RetainedEntitySelection { .. } => None,
        }
    }
}
