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
        native_reference_invariants: &worth_query_installation::facade::WorthQueryInvariantExecutionContract,
    ) -> Result<Self, &'static str> {
        let mut closure = WorthQueryProviderPlanDeclaredClosure::default();
        bind_direct_role_closure(
            &mut closure,
            &effect_families(contracts.effects()),
            &invariant_slots(contracts.invariants()),
        );
        bind_application_candidate_effect_closure(
            &mut closure,
            posture,
            !contracts.publication_touches().scopes().is_empty(),
            &effect_families(contracts.effects()),
            &invariant_slots(contracts.invariants()),
        );
        let mut invariant_requirements = if posture.includes_application() {
            contracts.invariant_execution().requirements().to_vec()
        } else {
            Vec::new()
        };
        bind_native_reference_invariants(
            &mut closure,
            &mut invariant_requirements,
            native_reference_invariants,
        )?;
        closure.canonicalize();
        Ok(Self {
            direct: [("primary".to_owned(), closure)].into_iter().collect(),
            workflow: Default::default(),
            decision_fact_families: contracts.decision_facts().required_families().to_vec(),
            invariant_requirements,
            reconciliation_posture: "provisional-discard".to_owned(),
            application_graph_reads: Some(contracts.graph_reads().clone()),
            application_touches: Some(contracts.publication_touches().clone()),
            application_read_touch_overlap: Some(contracts.read_touch_overlap().clone()),
        })
    }
}

fn bind_native_reference_invariants(
    closure: &mut WorthQueryProviderPlanDeclaredClosure,
    requirements: &mut Vec<
        worth_query_installation::facade::WorthQueryInstalledInvariantExecutionRequirement,
    >,
    native: &worth_query_installation::facade::WorthQueryInvariantExecutionContract,
) -> Result<(), &'static str> {
    for requirement in native.requirements() {
        if let Some(existing) = requirements
            .iter()
            .find(|existing| existing.slot() == requirement.slot())
        {
            if existing != requirement {
                return Err("native reference invariant conflicts with the installed application requirement");
            }
        } else {
            requirements.push(requirement.clone());
        }
        closure.invariant.push(requirement.slot().to_owned());
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use worth_query_installation::facade::{
        WorthQueryInstalledInvariantExecutionRequirement, WorthQueryInvariantEnforcement,
        WorthQueryInvariantExecutionContract,
    };

    #[test]
    fn native_reference_obligation_survives_read_only_control_without_application_effects() {
        let requirement = WorthQueryInstalledInvariantExecutionRequirement::with_optional_bounds(
            "native-endpoint",
            "relational-custom-invariant",
            std::num::NonZeroU32::new(1).unwrap(),
            WorthQueryInvariantEnforcement::Blocking,
            "primary",
            ["application-proposed-state"],
            None,
            None,
        )
        .unwrap();
        let native = WorthQueryInvariantExecutionContract::declared([requirement.clone()]).unwrap();
        let mut declarations =
            WorthQueryProviderPlanDeclarations::test_direct("primary", None, false);
        let closure = declarations.direct.get_mut("primary").unwrap();
        bind_application_candidate_effect_closure(
            closure,
            WorthQueryApplicationEffectPosture::Platform,
            false,
            &["payment".into()],
            &["payment-rule".into()],
        );
        bind_native_reference_invariants(
            closure,
            &mut declarations.invariant_requirements,
            &native,
        )
        .unwrap();
        assert!(
            closure.touch.is_empty(),
            "native reference duties grant no application write scope"
        );
        assert_eq!(closure.effect, ["mutation"]);
        assert_eq!(closure.invariant, ["native-endpoint"]);
        assert_eq!(
            declarations.invariant_requirements_for(None, "primary"),
            [requirement]
        );
        assert!(declarations
            .invariant_requirements_for(None, "foreign-role")
            .is_empty());
        declarations
            .direct
            .get_mut("primary")
            .unwrap()
            .effect
            .clear();
        assert!(
            declarations
                .invariant_requirements_for(None, "primary")
                .is_empty(),
            "read-only execution alone cannot acquire mutation invariant duties"
        );
    }

    #[test]
    fn native_reference_obligation_deduplicates_exact_meaning_and_refuses_conflicting_slot() {
        let requirement = |version| {
            WorthQueryInstalledInvariantExecutionRequirement::with_optional_bounds(
                "native-endpoint",
                "relational-custom-invariant",
                std::num::NonZeroU32::new(version).unwrap(),
                WorthQueryInvariantEnforcement::Blocking,
                "primary",
                ["application-proposed-state"],
                None,
                None,
            )
            .unwrap()
        };
        let original = requirement(1);
        let mut requirements = vec![original.clone()];
        let mut closure = WorthQueryProviderPlanDeclaredClosure::default();
        let exact = WorthQueryInvariantExecutionContract::declared([original.clone()]).unwrap();
        bind_native_reference_invariants(&mut closure, &mut requirements, &exact).unwrap();
        assert_eq!(requirements, std::slice::from_ref(&original));
        let conflict = WorthQueryInvariantExecutionContract::declared([requirement(2)]).unwrap();
        assert!(
            bind_native_reference_invariants(&mut closure, &mut requirements, &conflict).is_err()
        );
        assert_eq!(requirements, [original]);
    }
}
