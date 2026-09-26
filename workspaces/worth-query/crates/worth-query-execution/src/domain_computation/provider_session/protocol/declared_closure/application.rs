use super::{
    bind_direct_role_closure, effect_families, invariant_slots, WorthQueryProviderPlanDeclarations,
    WorthQueryProviderPlanDeclaredClosure,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryApplicationEffectPosture {
    Application,
    Platform,
    ApplicationWithPlatform,
}

impl WorthQueryApplicationEffectPosture {
    pub(in crate::domain_computation) const fn includes_platform(self) -> bool {
        !matches!(self, Self::Application)
    }

    const fn includes_application(self) -> bool {
        !matches!(self, Self::Platform)
    }
}

impl WorthQueryProviderPlanDeclarations {
    pub(crate) fn from_application_contracts(
        contracts: &worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
        posture: WorthQueryApplicationEffectPosture,
    ) -> Self {
        let mut closure = WorthQueryProviderPlanDeclaredClosure::default();
        bind_direct_role_closure(
            &mut closure,
            &effect_families(contracts.effects()),
            &invariant_slots(contracts.invariants()),
        );
        bind_application_candidate_effect_closure(
            &mut closure,
            posture,
            !contracts.touches().scopes().is_empty(),
            &effect_families(contracts.effects()),
            &invariant_slots(contracts.invariants()),
        );
        closure.canonicalize();
        Self {
            direct: [("primary".to_owned(), closure)].into_iter().collect(),
            workflow: Default::default(),
            decision_fact_families: contracts.decision_facts().required_families().to_vec(),
            invariant_requirements: if posture.includes_application() {
                contracts.invariant_execution().requirements().to_vec()
            } else {
                Vec::new()
            },
            reconciliation_posture: "provisional-discard".to_owned(),
            application_graph_reads: Some(contracts.graph_reads().clone()),
            application_touches: Some(contracts.touches().clone()),
            application_read_touch_overlap: Some(contracts.read_touch_overlap().clone()),
        }
    }
}

pub(crate) fn bind_application_candidate_effect_closure(
    closure: &mut WorthQueryProviderPlanDeclaredClosure,
    posture: WorthQueryApplicationEffectPosture,
    touches_application: bool,
    application_effects: &[String],
    application_invariants: &[String],
) {
    closure.effect.clear();
    closure.invariant.clear();
    if posture.includes_application() && touches_application {
        closure.effect.extend_from_slice(application_effects);
        closure.invariant.extend_from_slice(application_invariants);
    }
    if posture.includes_platform() {
        closure.effect.push("mutation".to_owned());
    }
}
