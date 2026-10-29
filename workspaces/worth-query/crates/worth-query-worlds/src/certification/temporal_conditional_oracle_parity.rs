pub use crate::facade::certification::adapters;
pub use crate::facade::certification::contract;
pub use crate::facade::certification::courtroom_support;
pub use crate::facade::certification::schema;
pub use crate::facade::certification::test_request_policy;
pub use crate::facade::certification::world;

pub use courtroom_support::{assert_authoritative_value, observe};
pub use schema::IntentEffectField;
pub use world::CourtroomWorld;
pub use worth_query::facade::{
    certification::internal_conditional_outcome_for_certification,
    installed::conditional::WorthQueryConditionalOutcomeClass as InternalOutcome,
};
pub use worth_query_host::facade::primary_graph::{
    WorthQueryConditionalClockObservationOutcome as ClockOutcome,
    WorthQueryConditionalExecutionTerminal as Terminal,
    WorthQueryConditionalRuntimeInstallationDenialKind as InstallationDenial,
    WorthQueryConditionalSignalDecision as SignalDecision,
};
pub use worth_signal::facade::SignalConditionalDecisionClass as SignalClass;

#[derive(Clone, Copy)]
pub enum Case {
    Satisfied,
    Unsatisfied,
    Failed,
    Future,
    Due,
    Cancelled,
    Superseded,
    Completed,
    DuplicateClock,
    ReorderedClock,
    ProviderReplaced,
    GenerationChanged,
}

impl Case {
    pub const ALL: [Self; 12] = [
        Self::Satisfied,
        Self::Unsatisfied,
        Self::Failed,
        Self::Future,
        Self::Due,
        Self::Cancelled,
        Self::Superseded,
        Self::Completed,
        Self::DuplicateClock,
        Self::ReorderedClock,
        Self::ProviderReplaced,
        Self::GenerationChanged,
    ];
}

pub fn assert_case(request_execution: &worth_execution::ExecutionRequest<'_, '_>, case: Case) {
    match case {
        Case::Satisfied | Case::Due => {
            assert_computed_changed(request_execution, CourtroomWorld::publish("ready"))
        }
        Case::Unsatisfied => {
            let world = CourtroomWorld::publish("blocked");
            let receipt = observe(request_execution, &world);
            assert_oracle(SignalClass::SuppressedBeforeCompute, &receipt);
            assert_eq!(world.contacts.snapshot(), (1, 0, 0, 0));
        }
        Case::Failed => {
            let world = CourtroomWorld::publish("ready");
            world.predicate_panic.set(true);
            let receipt = observe(request_execution, &world);
            assert_eq!(receipt.failed_operation_count(), 0);
            let [lineage] = receipt.execution_provenance() else {
                panic!("failed lineage")
            };
            assert_eq!(lineage.signal_decision(), None);
            assert_eq!(lineage.terminal(), Terminal::Failed);
        }
        Case::Future => {
            let mut world = CourtroomWorld::publish("ready");
            world.clock_control.push(1, 4);
            assert_no_effect(request_execution, &mut world);
        }
        Case::Cancelled | Case::Completed => {
            let mut world = CourtroomWorld::publish("ready");
            let lifecycle = if matches!(case, Case::Cancelled) {
                "cancelled"
            } else {
                "completed"
            };
            world.amend_intent(2, lifecycle, "ready");
            assert_no_effect(request_execution, &mut world);
        }
        Case::Superseded => {
            let mut world = CourtroomWorld::publish("ready");
            world.supersede_intent(2, 5, "active", "successor", "ready");
            assert_computed_changed(request_execution, world);
        }
        Case::DuplicateClock => {
            let world = CourtroomWorld::publish("ready");
            let product = world.application.current_world();
            let mut clock = world
                .application
                .with_host_advancement(|host_phase| {
                    world
                        .application
                        .on_branch(product)
                        .select(&host_phase)
                        .unwrap()
                        .conditional_clock(
                            &host_phase
                                .checked_execution_request_for(world.application.product_runtime())
                                .expect("phase belongs to application"),
                            &world.clock,
                        )
                        .unwrap()
                })
                .expect("installed host call opens");
            let ClockOutcome::Accepted(receipt) = clock.observe(request_execution) else {
                panic!("the first exact-lane observation must be accepted")
            };
            assert_oracle(SignalClass::ComputedChanged, &receipt);
            world.clock_control.push(1, 10);
            assert!(matches!(
                clock.observe(request_execution),
                ClockOutcome::Duplicate(_)
            ));
        }
        Case::ReorderedClock => {
            let world = CourtroomWorld::publish("ready");
            let product = world.application.current_world();
            let mut clock = world
                .application
                .with_host_advancement(|host_phase| {
                    world
                        .application
                        .on_branch(product)
                        .select(&host_phase)
                        .unwrap()
                        .conditional_clock(
                            &host_phase
                                .checked_execution_request_for(world.application.product_runtime())
                                .expect("phase belongs to application"),
                            &world.clock,
                        )
                        .unwrap()
                })
                .expect("installed host call opens");
            let ClockOutcome::Accepted(receipt) = clock.observe(request_execution) else {
                panic!("the first exact-lane observation must be accepted")
            };
            assert_oracle(SignalClass::ComputedChanged, &receipt);
            world.clock_control.push(2, 9);
            assert!(matches!(
                clock.observe(request_execution),
                ClockOutcome::Reordered
            ));
        }
        Case::ProviderReplaced => assert_provider_replacement(request_execution),
        Case::GenerationChanged => {
            let mut world = CourtroomWorld::publish("ready");
            let successor = std::sync::Arc::new(world.installation.successor_generation());
            let product = world.application.current_world();
            assert_eq!(
                world
                    .application
                    .reinstall_conditional_runtime_for_installation(
                        request_execution,
                        successor,
                        product
                    )
                    .unwrap_err()
                    .kind(),
                InstallationDenial::RebindRequired
            );
        }
    }
}

pub fn assert_computed_changed(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: CourtroomWorld,
) {
    let receipt = observe(request_execution, &world);
    assert_oracle(SignalClass::ComputedChanged, &receipt);
    assert_authoritative_value(&world, IntentEffectField::reference(), {
        let [lineage] = receipt.execution_provenance() else {
            panic!("committed lineage")
        };
        if lineage.intent_revision() == 1 {
            "payload"
        } else {
            "successor"
        }
        .to_string()
    });
}

pub fn assert_no_effect(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut CourtroomWorld,
) {
    let receipt = observe(request_execution, world);
    assert_eq!(receipt.committed_operation_count(), 0);
    assert!(receipt.execution_provenance().is_empty());
    assert_authoritative_value(world, IntentEffectField::reference(), "pending".to_string());
}

pub fn assert_oracle(
    signal_class: SignalClass,
    receipt: &worth_query_host::facade::primary_graph::WorthQueryConditionalClockObservationReceipt<
        adapters::CourtroomClock,
    >,
) {
    let oracle = internal_conditional_outcome_for_certification(signal_class);
    let [lineage] = receipt.execution_provenance() else {
        panic!("one oracle lineage")
    };
    match oracle {
        InternalOutcome::ComputedChanged => {
            assert_eq!(lineage.signal_decision(), Some(SignalDecision::Eligible));
            assert_eq!(lineage.terminal(), Terminal::Committed);
            assert_eq!(receipt.committed_operation_count(), 1);
        }
        InternalOutcome::Suppressed => {
            assert_eq!(lineage.signal_decision(), Some(SignalDecision::Suppressed));
            assert_eq!(lineage.terminal(), Terminal::SuppressedRetained);
            assert_eq!(receipt.committed_operation_count(), 0);
        }
        _ => panic!("this temporal court admits only changed or suppressed oracle effects"),
    }
}

pub fn assert_provider_replacement(request_execution: &worth_execution::ExecutionRequest<'_, '_>) {
    let incumbent = CourtroomWorld::publish("ready");
    let replacement = CourtroomWorld::publish_replacement("ready");
    assert!(replacement
        .application
        .with_host_advancement(|host_phase| replacement
            .application
            .on_branch(replacement.application.current_world())
            .select(&host_phase)
            .unwrap()
            .conditional_clock(request_execution, &incumbent.clock)
            .is_err())
        .expect("installed host call opens"));
    drop(incumbent);
    assert_computed_changed(request_execution, replacement);
}

#[path = "temporal_conditional_oracle_parity/managed_cleanup.rs"]
pub mod managed_cleanup;
