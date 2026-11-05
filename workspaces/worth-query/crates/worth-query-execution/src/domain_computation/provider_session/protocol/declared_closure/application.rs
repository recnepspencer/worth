use {crate::domain_computation::vocabulary::provider_session::declared_closure::bind_direct_role_closure, crate::domain_computation::vocabulary::provider_session::declared_closure::effect_families, crate::domain_computation::vocabulary::provider_session::declared_closure::invariant_slots, crate::domain_computation::vocabulary::provider_session::declared_closure::WorthQueryProviderPlanDeclarations, crate::domain_computation::vocabulary::provider_session::declared_closure::WorthQueryProviderPlanDeclaredClosure};






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
