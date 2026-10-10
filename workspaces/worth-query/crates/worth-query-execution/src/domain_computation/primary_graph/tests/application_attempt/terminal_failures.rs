use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use super::{
    admitted_mutation_free_program, admitted_program, authenticated_principal, idempotency,
    installed_authorization_world, live_scope, resolved_account,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome,
};
use crate::facade::primary_graph::{
    WorthQueryExternalDispatchRequest, WorthQueryExternalEffectTransport,
    WorthQueryExternalTransportOutcome,
};

struct ZeroInvariantCompletingTransport(AtomicUsize);

impl WorthQueryExternalEffectTransport for ZeroInvariantCompletingTransport {
    fn dispatch(
        &self,
        _request: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        self.0.fetch_add(1, Ordering::AcqRel);
        WorthQueryExternalTransportOutcome::Completed
    }
}

fn install_zero_invariant_transport(
    world: &super::super::fixture::AuthorizationWorld,
) -> Arc<ZeroInvariantCompletingTransport> {
    let transport = Arc::new(ZeroInvariantCompletingTransport(AtomicUsize::new(0)));
    world
        .application
        .install_external_effect_transport(transport.clone())
        .expect("zero-invariant fixture installs its external transport");
    transport
}

#[path = "terminal_failures/preparation_rejection_is_denied_without_effect_or_idempotency_re.rs"]
mod preparation_rejection_is_denied_without_effect_or_idempotency_re;

#[path = "terminal_failures/pretransaction_commit_failure_is_proved_aborted_and_applies_noth.rs"]
mod pretransaction_commit_failure_is_proved_aborted_and_applies_noth;

#[path = "terminal_failures/causal_fact_survives_post_commit_snapshot_failure_via_relational.rs"]
mod causal_fact_survives_post_commit_snapshot_failure_via_relational;

#[path = "terminal_failures/idempotency_without_the_claimed_causal_fact_is_not_equivalent.rs"]
mod idempotency_without_the_claimed_causal_fact_is_not_equivalent;

#[test]
fn missing_owner_candidate_is_denied_before_semantic_invariant_execution() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let rejected = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "must-not-bypass-relational",
            )
        })
        .expect("the fixture read owns its installed host call");

    world.faults.skip_next_invariant_owner_execution();
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            rejected,
            idempotency(22, 22),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("missing owner admission must deny before semantic execution");
    };
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    assert_eq!(
        world
            .application
            .primary_provider
            .published_application_commit_count(),
        0,
        "owner validation denial must precede commit and outbox publication"
    );
    let _still_open = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &live_scope())
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn relational_invariant_violation_denies_before_provider_commit() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let rejected = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "must-not-pass-relational-invariant",
            )
        })
        .expect("the fixture read owns its installed host call");

    world.faults.violate_next_relational_invariant();
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            rejected,
            idempotency(23, 23),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the installed Relational invariant violation must deny");
    };
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    let _still_open = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &live_scope())
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn zero_semantic_invariants_still_require_relational_candidate_validation() {
    let world = installed_authorization_world(true);
    let transport = install_zero_invariant_transport(&world);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let rejected = world
        .application
        .with_host_advancement(|phase| {
            admitted_mutation_free_program(&phase, &world, &principal, &account, &request)
        })
        .expect("the fixture read owns its installed host call");

    world.faults.violate_next_relational_invariant();
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            rejected,
            idempotency(25, 25),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("zero semantic requirements must not bypass Relational validation");
    };
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    assert_eq!(
        world
            .application
            .primary_provider
            .published_application_commit_count(),
        0,
        "zero-requirement owner denial must precede commit and outbox publication"
    );
    assert_eq!(transport.0.load(Ordering::Acquire), 0);
}

#[test]
fn zero_semantic_invariants_commit_after_owner_candidate_admission() {
    let world = installed_authorization_world(true);
    let transport = install_zero_invariant_transport(&world);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            admitted_mutation_free_program(&phase, &world, &principal, &account, &request)
        })
        .expect("the fixture read owns its installed host call");

    let WorthQueryApplicationCommitOutcome::Committed(receipt) =
        world.application.compare_and_commit_application(
            program,
            idempotency(26, 26),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("owner-admitted candidate with no semantic requirements must commit");
    };
    assert_eq!(
        receipt.changed_record_count(),
        2,
        "only provider idempotency and outbox records may change"
    );
    assert_eq!(receipt.emitted_effect_count(), 1);
    assert!(
        receipt.dispatch_outbox().is_some(),
        "the mutation-free external effect must co-commit its outbox"
    );
    assert_eq!(transport.0.load(Ordering::Acquire), 1);
}

#[test]
fn owner_validated_application_touch_outside_installed_ceiling_is_denied() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let rejected = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "must-not-pass-installed-touch-ceiling",
            )
        })
        .expect("the fixture read owns its installed host call");

    world.faults.add_next_undeclared_application_touch();
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            rejected,
            idempotency(24, 24),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("an undeclared performed application touch must deny");
    };
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    let _still_open = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &live_scope())
        })
        .expect("the fixture read owns its installed host call");
}
