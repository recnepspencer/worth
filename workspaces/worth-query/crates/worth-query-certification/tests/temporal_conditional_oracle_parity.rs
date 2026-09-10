#[path = "../../worth-query-host/tests/temporal_conditional_operation/adapters.rs"]
mod adapters;
#[path = "../../worth-query-host/tests/temporal_conditional_operation/contract.rs"]
mod contract;
#[allow(dead_code)]
#[path = "../../worth-query-host/tests/temporal_conditional_operation/courtroom_support.rs"]
mod courtroom_support;
#[path = "../../worth-query-host/tests/temporal_conditional_operation/schema.rs"]
mod schema;
#[allow(dead_code)]
#[path = "../../worth-query-host/tests/temporal_conditional_operation/world.rs"]
mod world;

use courtroom_support::{assert_authoritative_value, observe};
use schema::IntentEffectField;
use world::CourtroomWorld;
use worth_query::facade::{
    certification::internal_conditional_outcome_for_certification,
    installed::conditional::WorthQueryConditionalOutcomeClass as InternalOutcome,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryConditionalClockObservationOutcome as ClockOutcome,
    WorthQueryConditionalExecutionTerminal as Terminal,
    WorthQueryConditionalRuntimeInstallationDenialKind as InstallationDenial,
    WorthQueryConditionalSignalDecision as SignalDecision,
};
use worth_signal::facade::SignalConditionalDecisionClass as SignalClass;

#[derive(Clone, Copy)]
enum Case {
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
    const ALL: [Self; 12] = [
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

#[test]
fn host_and_internal_conditional_oracle_agree_across_phase_nine_matrix() {
    for case in Case::ALL {
        assert_case(case);
    }
}

#[test]
fn internal_oracle_retains_all_seven_signal_classes() {
    for (class, expected) in [
        (
            SignalClass::ComputedChanged,
            InternalOutcome::ComputedChanged,
        ),
        (
            SignalClass::ComputedRevertedClean,
            InternalOutcome::ComputedRevertedClean,
        ),
        (
            SignalClass::DependencyUnchanged,
            InternalOutcome::DependencyUnchanged,
        ),
        (
            SignalClass::SuppressedBeforeCompute,
            InternalOutcome::Suppressed,
        ),
        (
            SignalClass::DeferredByCondition,
            InternalOutcome::DeferredByCondition,
        ),
        (
            SignalClass::DeferredTemporal,
            InternalOutcome::DeferredTemporal,
        ),
        (
            SignalClass::DeferredOnDemand,
            InternalOutcome::DeferredOnDemand,
        ),
    ] {
        assert_eq!(
            internal_conditional_outcome_for_certification(class),
            expected
        );
    }
}

#[test]
fn temporal_conditional_identity_has_no_private_digest_engine() {
    let owner = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../worth-query-execution/src/domain_computation/primary_graph/conditional_operation",
    );
    let mut pending = vec![owner];
    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().and_then(std::ffi::OsStr::to_str) == Some("rs") {
                let source = std::fs::read_to_string(&path).unwrap();
                assert!(!source.contains("use sha2"), "{}", path.display());
                assert!(!source.contains("Sha256::"), "{}", path.display());
            }
        }
    }
}

fn assert_case(case: Case) {
    match case {
        Case::Satisfied | Case::Due => assert_computed_changed(CourtroomWorld::publish("ready")),
        Case::Unsatisfied => {
            let mut world = CourtroomWorld::publish("blocked");
            let receipt = observe(&mut world);
            assert_oracle(SignalClass::SuppressedBeforeCompute, &receipt);
            assert_eq!(world.contacts.snapshot(), (1, 0, 0, 0));
        }
        Case::Failed => {
            let mut world = CourtroomWorld::publish("ready");
            world.predicate_panic.set(true);
            let receipt = observe(&mut world);
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
            assert_no_effect(&mut world);
        }
        Case::Cancelled | Case::Completed => {
            let mut world = CourtroomWorld::publish("ready");
            let lifecycle = if matches!(case, Case::Cancelled) {
                "cancelled"
            } else {
                "completed"
            };
            world.amend_intent(2, lifecycle, "ready");
            assert_no_effect(&mut world);
        }
        Case::Superseded => {
            let mut world = CourtroomWorld::publish("ready");
            world.supersede_intent(2, 5, "active", "successor", "ready");
            assert_computed_changed(world);
        }
        Case::DuplicateClock => {
            let world = CourtroomWorld::publish("ready");
            let product = world.application.current_world();
            let mut clock = world
                .application
                .on_branch(product)
                .select()
                .unwrap()
                .conditional_clock(&world.clock)
                .unwrap();
            let ClockOutcome::Accepted(receipt) = clock.observe() else {
                panic!("the first exact-lane observation must be accepted")
            };
            assert_oracle(SignalClass::ComputedChanged, &receipt);
            world.clock_control.push(1, 10);
            assert!(matches!(clock.observe(), ClockOutcome::Duplicate(_)));
        }
        Case::ReorderedClock => {
            let world = CourtroomWorld::publish("ready");
            let product = world.application.current_world();
            let mut clock = world
                .application
                .on_branch(product)
                .select()
                .unwrap()
                .conditional_clock(&world.clock)
                .unwrap();
            let ClockOutcome::Accepted(receipt) = clock.observe() else {
                panic!("the first exact-lane observation must be accepted")
            };
            assert_oracle(SignalClass::ComputedChanged, &receipt);
            world.clock_control.push(2, 9);
            assert!(matches!(clock.observe(), ClockOutcome::Reordered));
        }
        Case::ProviderReplaced => assert_provider_replacement(),
        Case::GenerationChanged => {
            let mut world = CourtroomWorld::publish("ready");
            let successor = std::sync::Arc::new(world.installation.successor_generation());
            let product = world.application.current_world();
            assert_eq!(
                world
                    .application
                    .reinstall_conditional_runtime_for_installation(successor, product)
                    .unwrap_err()
                    .kind(),
                InstallationDenial::RebindRequired
            );
        }
    }
}

fn assert_computed_changed(mut world: CourtroomWorld) {
    let receipt = observe(&mut world);
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

fn assert_no_effect(world: &mut CourtroomWorld) {
    let receipt = observe(world);
    assert_eq!(receipt.committed_operation_count(), 0);
    assert!(receipt.execution_provenance().is_empty());
    assert_authoritative_value(world, IntentEffectField::reference(), "pending".to_string());
}

fn assert_oracle(
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

fn assert_provider_replacement() {
    let incumbent = CourtroomWorld::publish("ready");
    let mut replacement = CourtroomWorld::publish_replacement("ready");
    assert!(replacement
        .application
        .on_branch(replacement.application.current_world())
        .select()
        .unwrap()
        .conditional_clock(&incumbent.clock)
        .is_err());
    drop(incumbent);
    assert_computed_changed(replacement);
}
