use std::sync::Arc;

use super::*;
use crate::facade::{
    ProductBranchCreationIntent, ProductBranchCreationPlans, RelationalBranchCreationPlan,
    RuntimeWorldConditionalDefinitionPublicationOutcome, SignalBranchCreationPlan,
};
use worth_runtime_bridge::facade::{
    BridgeInstalledConditionalLowering, BridgeSealedRuntimeAssembly,
};

#[path = "definition_publication/fixture.rs"]
mod fixture;
use fixture::{definition_world, source_free_request};

#[path = "definition_publication/recovery.rs"]
mod recovery;
#[path = "definition_publication/resource_budget.rs"]
mod resource_budget;

#[test]
fn performed_definition_successor_isolated_from_sibling_product_and_pinned_generation() {
    let fixture = definition_world();
    let cancellation = RuntimeWorldCancellationSource::new();
    let sibling = match fixture
        .owner
        .branch_port()
        .create_product_branch(
            fixture.root.clone(),
            ProductBranchCreationIntent::from_source(
                "definition-sibling",
                ProductBranchCreationPlans::new(
                    RelationalBranchCreationPlan::ReuseExact,
                    SignalBranchCreationPlan::ForkExact {
                        target: worth_signal::facade::branch::validate_signal_branch_name(
                            "definition-sibling-signal",
                        )
                        .unwrap(),
                    },
                ),
            )
            .unwrap(),
            &cancellation.token(),
        )
        .unwrap()
    {
        crate::lifecycle::RuntimeWorldBranchCreationOutcome::Performed(observation) => observation,
        other => panic!("sibling product creation must perform: {other:?}"),
    };
    let sibling_binding = fixture
        .bridge
        .admit_conditional_signal_basis(&fixture.lowering, sibling.basis().signal_basis())
        .expect("the sibling product admits its exact D0 definition");
    let pinned = fixture
        .bridge
        .admit_conditional_evaluation(
            worth_runtime_bridge::facade::BridgeConditionalEvaluationAdmissionRequest::source_free_at_signal_basis(
                &sibling_binding,
            ),
        )
        .expect("the sibling's D0 session is admitted before the successor publication");
    let predecessor_generation = fixture.lowering.signal_definition_generation();
    let prepared_world = fixture
        .owner
        .publication_port()
        .prepare_with_signal(
            fixture.root.clone(),
            CompositePublicationIntent::with_signal(None),
            &cancellation.token(),
            None,
        )
        .unwrap();
    let root_binding = fixture
        .bridge
        .admit_conditional_signal_basis(&fixture.lowering, fixture.root.basis().signal_basis())
        .unwrap();
    let prepared_bridge = fixture
        .bridge
        .prepare_owned_conditional_definition_successor(&root_binding, source_free_request(2))
        .unwrap();
    let next = match fixture
        .owner
        .publication_port()
        .publish_bridge_conditional_definition(
            prepared_world,
            prepared_bridge,
            &fixture.bridge,
            &cancellation.token(),
        ) {
        RuntimeWorldConditionalDefinitionPublicationOutcome::Performed {
            publication,
            lowering,
        } => (publication, lowering),
        _ => panic!("definition successor must perform"),
    };
    let current = fixture
        .owner
        .observation_port()
        .observe_product_branch(fixture.root.branch_identity())
        .unwrap();

    assert_eq!(
        fixture.lowering.signal_definition_generation(),
        predecessor_generation
    );
    assert_eq!(
        next.1.signal_definition_generation(),
        predecessor_generation + 1
    );
    assert_ne!(
        sibling.basis().signal_basis().observation(),
        fixture.root.basis().signal_basis().observation()
    );
    assert_ne!(
        current.basis().signal_basis().observation(),
        fixture.root.basis().signal_basis().observation()
    );
    let stale_d0 = fixture
        .bridge
        .admit_conditional_signal_basis(&fixture.lowering, current.basis().signal_basis())
        .err()
        .expect("the D0 lowering cannot bind the published D1 Signal basis");
    assert_eq!(
        stale_d0.kind(),
        worth_runtime_bridge::facade::BridgeConditionalDenialKind::StaleLowering
    );
    let sibling_after = fixture
        .owner
        .observation_port()
        .observe_product_branch(sibling.branch_identity())
        .unwrap();
    assert_eq!(
        sibling_after.basis().signal_basis().observation(),
        sibling.basis().signal_basis().observation()
    );
    let premature_d1 = fixture
        .bridge
        .admit_conditional_signal_basis(&next.1, sibling_after.basis().signal_basis())
        .err()
        .expect("the D1 lowering cannot bind the sibling's retained D0 Signal basis");
    assert_eq!(
        premature_d1.kind(),
        worth_runtime_bridge::facade::BridgeConditionalDenialKind::StaleLowering
    );
    execute_source_free(&fixture.bridge, &fixture.lowering, &pinned, 1);
    let successor_binding = fixture
        .bridge
        .admit_conditional_signal_basis(&next.1, current.basis().signal_basis())
        .unwrap();
    let successor_session = fixture
        .bridge
        .admit_conditional_evaluation(
            worth_runtime_bridge::facade::BridgeConditionalEvaluationAdmissionRequest::source_free_at_signal_basis(
                &successor_binding,
            ),
        )
        .unwrap();
    execute_source_free(&fixture.bridge, &next.1, &successor_session, 2);

    let sibling_world = fixture
        .owner
        .publication_port()
        .prepare_with_signal(
            sibling_after.clone(),
            CompositePublicationIntent::with_signal(None),
            &cancellation.token(),
            None,
        )
        .unwrap();
    let sibling_bridge = fixture
        .bridge
        .prepare_owned_conditional_definition_successor(&sibling_binding, source_free_request(3))
        .expect("B prepares from its retained D0 port after A publishes D1");
    let sibling_next = match fixture
        .owner
        .publication_port()
        .publish_bridge_conditional_definition(
            sibling_world,
            sibling_bridge,
            &fixture.bridge,
            &cancellation.token(),
        ) {
        RuntimeWorldConditionalDefinitionPublicationOutcome::Performed {
            publication,
            lowering,
        } => (publication, lowering),
        _ => panic!("B must independently publish from D0 after A/D1"),
    };
    let sibling_current = fixture
        .owner
        .observation_port()
        .observe_product_branch(sibling.branch_identity())
        .unwrap();
    let sibling_successor_binding = fixture
        .bridge
        .admit_conditional_signal_basis(&sibling_next.1, sibling_current.basis().signal_basis())
        .unwrap();
    let sibling_successor_session = fixture
        .bridge
        .admit_conditional_evaluation(
            worth_runtime_bridge::facade::BridgeConditionalEvaluationAdmissionRequest::source_free_at_signal_basis(
                &sibling_successor_binding,
            ),
        )
        .unwrap();
    execute_source_free(
        &fixture.bridge,
        &sibling_next.1,
        &sibling_successor_session,
        3,
    );
    drop(next.0.consume());
    drop(sibling_next.0.consume());
}

#[test]
fn stale_product_head_denies_definition_before_signal_or_bridge_visibility() {
    let fixture = definition_world();
    let lifecycle = fixture.bridge.conditional_lifecycle_probe();
    let cancellation = RuntimeWorldCancellationSource::new();
    let stale_world = fixture
        .owner
        .publication_port()
        .prepare_with_signal(
            fixture.root.clone(),
            CompositePublicationIntent::with_signal(None),
            &cancellation.token(),
            None,
        )
        .unwrap();
    let predecessor = fixture
        .bridge
        .admit_conditional_signal_basis(&fixture.lowering, fixture.root.basis().signal_basis())
        .unwrap();
    let stale_bridge = fixture
        .bridge
        .prepare_owned_conditional_definition_successor(&predecessor, source_free_request(2))
        .unwrap();
    let competing = fixture
        .owner
        .publication_port()
        .prepare_with_signal(
            fixture.root.clone(),
            CompositePublicationIntent::with_signal(None),
            &cancellation.token(),
            None,
        )
        .unwrap();
    assert!(matches!(
        fixture.owner.publication_port().execute_with_signal(
            competing,
            &mut (),
            &cancellation.token(),
            |_| Ok(())
        ),
        crate::facade::RuntimeWorldPublicationOutcome::Performed(_)
    ));

    let outcome = fixture
        .owner
        .publication_port()
        .publish_bridge_conditional_definition(
            stale_world,
            stale_bridge,
            &fixture.bridge,
            &cancellation.token(),
        );
    assert!(matches!(
        outcome,
        RuntimeWorldConditionalDefinitionPublicationOutcome::NoEffect(ref denial)
            if denial.cause() == crate::facade::NoEffectCause::StaleExpectedProductHead
    ));
    assert_eq!(fixture.lowering.signal_definition_generation(), 1);
    assert_eq!(fixture.bridge.installed_conditional_definition_count(), 1);
    let retention = lifecycle.bridge_conditional_retention().unwrap();
    assert_eq!(retention.retained_definition_candidates(), 0);
    assert_eq!(retention.retained_bytes(), 0);
}

#[test]
fn foreign_world_denies_definition_before_signal_or_bridge_visibility() {
    let source = definition_world();
    let source_lifecycle = source.bridge.conditional_lifecycle_probe();
    let foreign = definition_world();
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared_world = source
        .owner
        .publication_port()
        .prepare_with_signal(
            source.root.clone(),
            CompositePublicationIntent::with_signal(None),
            &cancellation.token(),
            None,
        )
        .unwrap();
    let predecessor = source
        .bridge
        .admit_conditional_signal_basis(&source.lowering, source.root.basis().signal_basis())
        .unwrap();
    let prepared_bridge = source
        .bridge
        .prepare_owned_conditional_definition_successor(&predecessor, source_free_request(2))
        .unwrap();

    let outcome = foreign
        .owner
        .publication_port()
        .publish_bridge_conditional_definition(
            prepared_world,
            prepared_bridge,
            &source.bridge,
            &cancellation.token(),
        );

    assert!(matches!(
        outcome,
        RuntimeWorldConditionalDefinitionPublicationOutcome::NoEffect(ref denial)
            if denial.cause() == crate::facade::NoEffectCause::OwnerDeniedBeforeEffect
    ));
    assert_eq!(source.bridge.installed_conditional_definition_count(), 1);
    assert_eq!(foreign.bridge.installed_conditional_definition_count(), 1);
    assert_eq!(source.lowering.signal_definition_generation(), 1);
    let current = source
        .owner
        .observation_port()
        .observe_product_branch(source.root.branch_identity())
        .unwrap();
    assert_eq!(current.snapshot(), source.root.snapshot());
    let retention = source_lifecycle.bridge_conditional_retention().unwrap();
    assert_eq!(retention.retained_definition_candidates(), 0);
    assert_eq!(retention.retained_bytes(), 0);
}

fn execute_source_free(
    bridge: &BridgeSealedRuntimeAssembly,
    lowering: &Arc<BridgeInstalledConditionalLowering>,
    session: &worth_runtime_bridge::facade::BridgeConditionalEvaluationSession,
    attempt: u64,
) {
    bridge
        .execute_admitted_conditional(
            session,
            worth_runtime_bridge::facade::BridgeConditionalExecutionRequest {
                lowering,
                query_binding_identity: "definition-generation-test",
                query_capability_identity: 1,
                snapshot_identity: "source-free",
                truth_branch_identity: None,
                bridge_snapshot_identity: None,
                execution_identity: "definition-generation-execution",
                attempt,
            },
            &mut (),
        )
        .expect("the exact retained definition generation remains executable");
}
