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
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitTerminalKind,
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

#[test]
fn preparation_rejection_is_denied_without_effect_or_idempotency_residue() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let rejected = admitted_program(
        &world,
        &principal,
        &account,
        &request,
        "prepared-replacement",
    );

    world.faults.reject_next_session_prepare();
    let WorthQueryApplicationCommitOutcome::Denied(denial) = world
        .application
        .compare_and_commit_application(rejected, idempotency(19, 19))
    else {
        panic!("provider preparation rejection must be a typed denial");
    };
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::ProviderPlan
    );
    let _still_open = resolved_account(&world, "open", &live_scope());

    let retry = admitted_program(
        &world,
        &principal,
        &account,
        &request,
        "prepared-replacement",
    );
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(retry, idempotency(19, 19)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
}

#[test]
fn pretransaction_commit_failure_is_proved_aborted_and_applies_nothing() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let rejected = admitted_program(&world, &principal, &account, &request, "atomic-replacement");

    world.faults.reject_next_commit_before_transaction();
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(rejected, idempotency(20, 20)),
        WorthQueryApplicationCommitOutcome::Aborted
    ));
    let _still_open = resolved_account(&world, "open", &live_scope());

    let retry = admitted_program(&world, &principal, &account, &request, "atomic-replacement");
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(retry, idempotency(20, 20)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
}

#[test]
fn index_publication_failure_recovers_the_committed_transaction_before_returning() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let first = admitted_program(&world, &principal, &account, &request, "index-replacement");
    let retry = admitted_program(&world, &principal, &account, &request, "index-replacement");

    world.faults.fail_next_index_publication();
    let WorthQueryApplicationCommitOutcome::Committed(first_receipt) = world
        .application
        .compare_and_commit_application(first, idempotency(21, 21))
    else {
        panic!("index reconstruction must prove the committed transaction");
    };
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) = world
        .application
        .compare_and_commit_application(retry, idempotency(21, 21))
    else {
        panic!("index reconstruction must recover the committed idempotency record");
    };
    assert!(receipt.is_same_authoritative_commit(&first_receipt));
    assert_eq!(
        first_receipt.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Executed
    );
    assert_eq!(
        receipt.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Recovered
    );
    assert!(receipt.changed_record_count() >= 2);
    let _committed = resolved_account(&world, "index-replacement", &live_scope());
}

#[test]
fn causal_fact_survives_index_publication_failure_via_relational_owner_read() {
    use crate::domain_computation::application_aftermath::{
        WorthQueryAftermathCausalRole, WorthQueryPendingAftermathCausality,
    };

    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let program = admitted_program(&world, &principal, &account, &request, "causal-replacement");
    let parent = world
        .selected_product()
        .product()
        .relational_basis()
        .observation()
        .commit_receipt()
        .cloned()
        .expect("fixture has an authoritative branch head");
    let pending = WorthQueryPendingAftermathCausality::undo_of(parent.clone());

    world.faults.fail_next_index_publication();
    let WorthQueryApplicationCommitOutcome::Committed(receipt) = world
        .application
        .compare_and_commit_application_with_aftermath(
            program,
            idempotency(29, 29),
            pending.clone(),
        )
    else {
        panic!("owner reconstruction must recover the causal commit");
    };
    let carried = receipt
        .aftermath_causality()
        .expect("committed receipt carries recovered causal fact");
    assert_eq!(carried.role(), WorthQueryAftermathCausalRole::Undo);
    assert_eq!(carried.parent(), &parent);
    assert_eq!(carried.child(), receipt.commit_reference());

    let selected = world.selected_product();
    let reread = world
        .application
        .primary_provider
        .resolve_aftermath_causality_at_basis(selected.product().relational_basis(), &pending, None)
        .expect("owner read succeeds")
        .expect("co-committed fact remains visible");
    assert_eq!(reread, *carried);
}

#[test]
fn idempotency_without_the_claimed_causal_fact_is_not_equivalent() {
    use crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality;

    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let first = admitted_program(&world, &principal, &account, &request, "plain-commit");
    let retry = admitted_program(&world, &principal, &account, &request, "plain-commit");
    let parent = world
        .selected_product()
        .product()
        .relational_basis()
        .observation()
        .commit_receipt()
        .cloned()
        .expect("fixture head");
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(first, idempotency(30, 30)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));

    let WorthQueryApplicationCommitOutcome::Denied(denial) = world
        .application
        .compare_and_commit_application_with_aftermath(
            retry,
            idempotency(30, 30),
            WorthQueryPendingAftermathCausality::undo_of(parent),
        )
    else {
        panic!("a plain idempotency row cannot impersonate a causal commit");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
    );
}

#[test]
fn missing_owner_candidate_is_denied_before_semantic_invariant_execution() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let rejected = admitted_program(
        &world,
        &principal,
        &account,
        &request,
        "must-not-bypass-relational",
    );

    world.faults.skip_next_invariant_owner_execution();
    let WorthQueryApplicationCommitOutcome::Denied(denial) = world
        .application
        .compare_and_commit_application(rejected, idempotency(22, 22))
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
    let _still_open = resolved_account(&world, "open", &live_scope());
}

#[test]
fn relational_invariant_violation_denies_before_provider_commit() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let rejected = admitted_program(
        &world,
        &principal,
        &account,
        &request,
        "must-not-pass-relational-invariant",
    );

    world.faults.violate_next_relational_invariant();
    let WorthQueryApplicationCommitOutcome::Denied(denial) = world
        .application
        .compare_and_commit_application(rejected, idempotency(23, 23))
    else {
        panic!("the installed Relational invariant violation must deny");
    };
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    let _still_open = resolved_account(&world, "open", &live_scope());
}

#[test]
fn zero_semantic_invariants_still_require_relational_candidate_validation() {
    let world = installed_authorization_world(true);
    let transport = install_zero_invariant_transport(&world);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let rejected = admitted_mutation_free_program(&world, &principal, &account, &request);

    world.faults.violate_next_relational_invariant();
    let WorthQueryApplicationCommitOutcome::Denied(denial) = world
        .application
        .compare_and_commit_application(rejected, idempotency(25, 25))
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
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let program = admitted_mutation_free_program(&world, &principal, &account, &request);

    let WorthQueryApplicationCommitOutcome::Committed(receipt) = world
        .application
        .compare_and_commit_application(program, idempotency(26, 26))
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
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let rejected = admitted_program(
        &world,
        &principal,
        &account,
        &request,
        "must-not-pass-installed-touch-ceiling",
    );

    world.faults.add_next_undeclared_application_touch();
    let WorthQueryApplicationCommitOutcome::Denied(denial) = world
        .application
        .compare_and_commit_application(rejected, idempotency(24, 24))
    else {
        panic!("an undeclared performed application touch must deny");
    };
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    let _still_open = resolved_account(&world, "open", &live_scope());
}
