use super::*;

#[test]
fn resource_attempt_requires_the_exact_binding_and_installed_contract() {
    crate::domain_computation::primary_graph::with_test_advancement(|_active_phase| {
        let runtime = runtime();
        let (foreign_binding, contract_identity) = admitted_plan("foreign-binding");
        let installed_authority = authority(
            &runtime,
            "installed-binding",
            &contract_identity,
            &foreign_binding,
        );

        let denial =
            match runtime.start_direct_resource_attempt(&installed_authority, foreign_binding) {
                Err(denial) => denial,
                Ok(_) => panic!("foreign resource-plan binding started an execution attempt"),
            };

        assert_eq!(
            denial.kind(),
            &WorthQueryExecutionResourceAdmissionDenialKind::ResourcePlanAuthorityMismatch
        );
        assert_eq!(denial.counters().capacity_reservation_checks, 0);
        assert_eq!(denial.counters().provider_session_mints, 0);

        let (foreign_contract, _) = admitted_plan("installed-binding");
        let foreign_contract_authority = authority(
            &runtime,
            "installed-binding",
            "another-contract",
            &foreign_contract,
        );
        let denial = match runtime
            .start_direct_resource_attempt(&foreign_contract_authority, foreign_contract)
        {
            Err(denial) => denial,
            Ok(_) => panic!("foreign resource contract started an execution attempt"),
        };
        assert_eq!(
            denial.kind(),
            &WorthQueryExecutionResourceAdmissionDenialKind::ResourcePlanAuthorityMismatch
        );
        assert_eq!(denial.counters().capacity_reservation_checks, 0);
        assert_eq!(denial.counters().provider_session_mints, 0);
    });
}

#[test]
fn exact_resource_plan_binding_can_start_one_direct_attempt() {
    let runtime = runtime();
    let (plan, contract_identity) = admitted_plan("installed-binding");
    let authority = authority(&runtime, "installed-binding", &contract_identity, &plan);

    let attempt = runtime
        .start_direct_resource_attempt(&authority, plan)
        .unwrap();

    assert_eq!(attempt.resources().counters().capacity_reservations, 1);
    assert_eq!(attempt.resources().counters().provider_session_mints, 1);
}

#[test]
fn stale_bound_operation_cannot_start_an_execution_attempt() {
    crate::domain_computation::primary_graph::with_test_advancement(|_active_phase| {
        let mut runtime = runtime();
        let (plan, contract_identity) = admitted_plan("installed-binding");
        let authority = authority(&runtime, "installed-binding", &contract_identity, &plan);
        runtime
            .commit_successor_installation(Arc::new(
                runtime.installed_packages().successor_generation(),
            ))
            .unwrap();

        let denial = match runtime.start_direct_resource_attempt(&authority, plan) {
            Err(denial) => denial,
            Ok(_) => panic!("stale bound operation started an execution attempt"),
        };

        assert_eq!(
        denial.kind(),
        &WorthQueryExecutionResourceAdmissionDenialKind::RuntimeAuthority(
            worth_query_installation::facade::WorthQueryDomainHandleDenialKind::StaleInstallationGeneration,
        )
    );
        assert_eq!(denial.counters().capacity_reservation_checks, 0);
        assert_eq!(denial.counters().capacity_reservations, 0);
        assert_eq!(denial.counters().provider_session_mints, 0);
    });
}

#[test]
fn resource_attempt_rejects_omitted_installed_support_participants() {
    crate::domain_computation::primary_graph::with_test_advancement(|_active_phase| {
        for (conditional_nodes, graph_providers, commit_groups) in [
            (&["operation:gate"][..], &[][..], &[][..]),
            (&[][..], &["geometry"][..], &[][..]),
            (&[][..], &[][..], &["geometry,labels"][..]),
        ] {
            let runtime = runtime();
            let (plan, contract_identity) = admitted_plan("installed-binding");
            let mut authority = authority(&runtime, "installed-binding", &contract_identity, &plan);
            authority.direct_resource_topology = topology(
                conditional_nodes.iter().copied(),
                graph_providers.iter().copied(),
                commit_groups.iter().copied(),
            );

            let denial = match runtime.start_direct_resource_attempt(&authority, plan) {
                Err(denial) => denial,
                Ok(_) => panic!("an omitted installed participant started an execution attempt"),
            };

            assert_eq!(
                denial.kind(),
                &WorthQueryExecutionResourceAdmissionDenialKind::ResourcePlanAuthorityMismatch
            );
            assert_eq!(denial.counters().capacity_reservation_checks, 0);
            assert_eq!(denial.counters().provider_session_mints, 0);
        }
    });
}

#[test]
fn resource_attempt_rejects_caller_reconstructed_support() {
    crate::domain_computation::primary_graph::with_test_advancement(|_active_phase| {
        let runtime = runtime();
        let (installed_plan, contract_identity) =
            admitted_plan_with_support_limit("installed-binding", 2);
        let (reconstructed_plan, reconstructed_contract_identity) =
            admitted_plan_with_support_limit("installed-binding", 8);
        assert_eq!(contract_identity, reconstructed_contract_identity);
        let authority = authority(
            &runtime,
            "installed-binding",
            &contract_identity,
            &installed_plan,
        );

        let denial = match runtime.start_direct_resource_attempt(&authority, reconstructed_plan) {
            Err(denial) => denial,
            Ok(_) => panic!("caller-reconstructed support started an execution attempt"),
        };

        assert_eq!(
            denial.kind(),
            &WorthQueryExecutionResourceAdmissionDenialKind::ResourcePlanAuthorityMismatch
        );
        assert_eq!(denial.counters().capacity_reservation_checks, 0);
        assert_eq!(denial.counters().provider_session_mints, 0);
    });
}

fn topology<'a>(
    conditional_nodes: impl Iterator<Item = &'a str>,
    graph_providers: impl Iterator<Item = &'a str>,
    commit_groups: impl Iterator<Item = &'a str>,
) -> WorthQueryExecutionResourceTopology {
    test_topology(conditional_nodes, graph_providers, commit_groups)
}
