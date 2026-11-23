//! Own the normalized inputs carried through commit preparation.
pub(crate) mod meaning;
pub(crate) mod retained_entity_payload;
pub(crate) use meaning::RelationalSemanticPlan;
pub(crate) use retained_entity_payload::PreparedRetainedEntityPayload;

#[derive(Debug)]
pub(crate) enum PreparedRelationalCommitPlan {
    Mutation(crate::mvcc::OwnedNormalizedPlan),
    OwnedGraphSelection {
        commands: crate::mvcc::OwnedNormalizedPlan,
        retained: crate::mvcc::owned_graph_selection::PreparedOwnedGraphSelection,
    },
    RetainedEntitySelection {
        // Native consequences may add commands; none stand in for selection.
        commands: crate::mvcc::OwnedNormalizedPlan,
        retained: PreparedRetainedEntityPayload,
    },
}

impl PreparedRelationalCommitPlan {
    pub(crate) fn view(&self) -> RelationalSemanticPlan<'_> {
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
            Self::Mutation(_) | Self::RetainedEntitySelection { .. } => None,
        }
    }
}
