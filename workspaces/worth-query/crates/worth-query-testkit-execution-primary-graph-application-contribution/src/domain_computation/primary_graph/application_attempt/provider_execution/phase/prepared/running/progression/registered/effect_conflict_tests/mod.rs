use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::registered::WorthQueryRegisteredProviderAttempt;
use crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_execution::facade::domain_computation::{
    WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryProvisionalDenialKind as Kind,
    WorthQueryProvisionalEffectAction, WorthQueryProvisionalEffectStep,
};

pub fn assert_conflicting_effect_program_is_rejected(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    registered: WorthQueryRegisteredProviderAttempt<'_>,
    world: &AuthorizationWorld,
) {
    let (staged, steps, _dispatch_outbox) = registered.into_parts_for_test();
    let control = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        None,
    );
    let receipt = (*(*world.application.runtime_for_test()).primary_provider_for_test())
        .capture_application_read_set_for_test(execution, &staged.read_authority(), control)
        .expect("real registered facts must produce a decision read set");
    let fresh = match (*(*world.application.runtime_for_test()).primary_provider_for_test())
        .compare_application_read_set_for_test(
            execution,
            &staged.read_authority(),
            receipt,
            control,
        )
        .expect("real registered facts must compare")
    {
        WorthQueryDecisionReadSetFreshnessOutcome::Fresh(fresh) => fresh,
        WorthQueryDecisionReadSetFreshnessOutcome::Stale(_) => {
            panic!("unmodified fixture facts must remain fresh")
        }
    };
    assert!(
        !steps.is_empty(),
        "the registered mutation must have a real effect"
    );
    let authority = staged.effect_authority();
    let step = &steps[0];
    let missing_artifact = step
        .clone()
        .with_artifact_dependencies(["undeclared-artifact"])
        .unwrap();
    let unresolved_symbol = step
        .clone()
        .with_symbolic_dependencies(["uncreated-object"])
        .unwrap();
    let unread_target = WorthQueryProvisionalEffectStep::new(
        step.effect_family(),
        WorthQueryProvisionalEffectAction::Replace {
            target_identity: "unobserved-record".into(),
        },
    )
    .unwrap();
    for (effect, expected) in [
        (missing_artifact, Kind::UndeclaredArtifactDependency),
        (unresolved_symbol, Kind::UnknownSymbolicReference),
        (unread_target, Kind::UndeclaredTarget),
    ] {
        let denied = authority
            .lower_shared_provisional_program_for_test(&fresh, [effect].into())
            .err()
            .expect("unclosed effects must be rejected against real decision facts");
        assert_eq!(denied.kind(), expected);
    }
    let lowered = staged
        .effect_authority()
        .lower_shared_provisional_program_for_test(&fresh, std::sync::Arc::from([]))
        .expect("an empty program is valid until matched to the registered mutation");
    let provider = world
        .application
        .runtime_for_test()
        .primary_provider_for_test();
    let denial = match provider.stage_commit_candidate(staged, fresh, lowered) {
        Ok(_) => panic!("a different program must not stage the registered attempt"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.detail(),
        "lowered effect program conflicts with registered application effects",
    );
    assert_eq!(
        world.application.provider_session_resource_count_for_test(),
        0
    );
}
