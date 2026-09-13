/// Prepared application ownership transition committed inside the runtime's
/// complete publication transaction.
pub(crate) enum WorthUiPreparedApplicationPublication {
    Replacement {
        successor: Box<crate::facade::WorthUiApp>,
        intent_contract: crate::declaration::UiIntentCatalogSemanticComparison,
        appearance_consumer_contract_unchanged: bool,
        projection_contract_unchanged: bool,
    },
    MountedGraph(
        Box<
            crate::facade::prepared_application_authority::WorthUiPreparedApplicationGraphSuccessor,
        >,
    ),
}

impl WorthUiPreparedApplicationPublication {
    pub(crate) fn replacement_authority(
        &self,
    ) -> Option<&crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority>
    {
        match self {
            Self::Replacement { successor, .. } => Some(successor.prepared_authority()),
            Self::MountedGraph(_) => None,
        }
    }

    pub(crate) fn replacement(
        predecessor: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        successor: crate::facade::WorthUiApp,
    ) -> Self {
        let intent_contract = predecessor
            .intent_catalog()
            .compare_semantic_contract(successor.prepared_authority().intent_catalog());
        let predecessor_demand = predecessor.consumed_fact_index();
        let successor_demand = successor.prepared_authority().consumed_fact_index();
        let appearance_consumer_contract_unchanged =
            predecessor_demand.has_same_appearance_consumer_contract(successor_demand);
        let previous_projection = predecessor.semantic_handoff();
        let next_projection = successor.prepared_authority().semantic_handoff();
        let projection_contract_unchanged = previous_projection.projection_requirements()
            == next_projection.projection_requirements()
            && previous_projection.projection_contents() == next_projection.projection_contents();
        Self::Replacement {
            successor: Box::new(successor),
            intent_contract,
            appearance_consumer_contract_unchanged,
            projection_contract_unchanged,
        }
    }

    pub(crate) fn mounted_graph(
        successor: crate::facade::prepared_application_authority::WorthUiPreparedApplicationGraphSuccessor,
    ) -> Self {
        Self::MountedGraph(Box::new(successor))
    }

    pub(super) fn permits_execution_plan_semantic_no_op(&self) -> bool {
        match self {
            Self::Replacement {
                intent_contract,
                appearance_consumer_contract_unchanged,
                projection_contract_unchanged,
                ..
            } => {
                *intent_contract
                    == crate::declaration::UiIntentCatalogSemanticComparison::Equivalent
                    && *appearance_consumer_contract_unchanged
                    && *projection_contract_unchanged
            }
            Self::MountedGraph(_) => true,
        }
    }

    pub(super) fn commit_once(self, active: &mut crate::facade::WorthUiApp) {
        match self {
            Self::Replacement { successor, .. } => *active = *successor,
            Self::MountedGraph(successor) => {
                active
                    .commit_graph_successor(*successor)
                    .expect("prepared mounted graph successor retains its predecessor");
            }
        }
    }
}
