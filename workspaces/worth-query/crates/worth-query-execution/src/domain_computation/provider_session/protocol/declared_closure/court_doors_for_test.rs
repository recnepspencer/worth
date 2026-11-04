//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProviderPlanDeclarations {
    #[doc(hidden)]
    pub fn test_direct_for_test(
        graph_role: &str,
        read_access: Option<WorthQueryOperationGraphAccess>,
        touched: bool,
    ) -> Self {
        Self::test_direct(graph_role, read_access, touched)
    }
}

impl WorthQueryProviderPlanDeclarations {
    #[doc(hidden)]
    pub fn test_direct_with_decision_facts_for_test(
        graph_role: &str,
        read_access: WorthQueryOperationGraphAccess,
        decision_fact_families: Vec<worth_query_installation::facade::WorthQueryDecisionFactFamily>,
    ) -> Self {
        Self::test_direct_with_decision_facts(graph_role, read_access, decision_fact_families)
    }
}

impl WorthQueryProviderPlanDeclarations {
    #[doc(hidden)]
    pub fn test_effect_with_decision_facts_for_test(
        graph_role: &str,
        decision_fact_families: Vec<worth_query_installation::facade::WorthQueryDecisionFactFamily>,
    ) -> Self {
        Self::test_effect_with_decision_facts(graph_role, decision_fact_families)
    }
}

impl WorthQueryProviderPlanDeclarations {
    #[doc(hidden)]
    pub fn test_effect_with_decision_facts_and_invariants_for_test(
        graph_role: &str,
        decision_fact_families: Vec<worth_query_installation::facade::WorthQueryDecisionFactFamily>,
        invariant_requirements: Vec<
            worth_query_installation::facade::WorthQueryInstalledInvariantExecutionRequirement,
        >,
    ) -> Self {
        Self::test_effect_with_decision_facts_and_invariants(
            graph_role,
            decision_fact_families,
            invariant_requirements,
        )
    }
}

impl WorthQueryProviderPlanDeclarations {
    #[doc(hidden)]
    pub fn test_workflow_stage_for_test(
        stage_identity: &str,
        graph_role: &str,
        read_access: WorthQueryOperationGraphAccess,
    ) -> Self {
        Self::test_workflow_stage(stage_identity, graph_role, read_access)
    }
}

impl WorthQueryProviderPlanDeclarations {
    #[doc(hidden)]
    pub fn decision_fact_families_for_test(
        &self,
    ) -> &[worth_query_installation::facade::WorthQueryDecisionFactFamily] {
        Self::decision_fact_families(self)
    }
}

impl WorthQueryProviderPlanDeclaredClosure {
    #[doc(hidden)]
    pub fn effect_for_test(&self) -> &Vec<String> {
        &self.effect
    }
}

impl WorthQueryProviderPlanDeclaredClosure {
    #[doc(hidden)]
    pub fn invariant_for_test(&self) -> &Vec<String> {
        &self.invariant
    }
}
