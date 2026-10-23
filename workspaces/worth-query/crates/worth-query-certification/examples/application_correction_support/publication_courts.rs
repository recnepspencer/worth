//! Public publication courts share the example's actual installed numeric model.
use super::{authentication::*, edit::Value, host, query::Row};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationCorrectionDenial, WorthQueryApplicationCorrectionPreparation,
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
        WorthQueryCorrectionHistoryLimits,
    },
    primary_graph::WorthQueryCorrectionDerivationDenial,
};
fn history_limits() -> WorthQueryCorrectionHistoryLimits {
    WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN)
}

pub fn reverse_inherited_fork_and_refuse_key_target_drift() {
    let host = host::install();
    let source = host::edit(&host, Value::Primary(11), "source");
    let fork = || {
        host.with_host_advancement(|phase| {
            host.runtime()
                .branches()
                .fork(host.runtime().current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap()
    };
    let first = fork();
    let second = fork();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope).on_branch(first);
    let selected = request
        .corrections()
        .select(&source, history_limits())
        .unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("shared-reverse")
        .limits(history_limits())
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("fork reversal must prepare");
    };
    let outcome = prepared.publish().unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("fork must publish: {outcome:?}");
    };
    assert_eq!(
        receipt.commit_reference().parents,
        vec![source.commit_reference().commit_id]
    );
    assert_ne!(
        receipt.commit_reference().branch_id,
        source.commit_reference().branch_id,
        "the inherited parent retains its original branch while the new child belongs to the fork"
    );
    assert_eq!(
        host::read_on(&host, first),
        [Row {
            primary: 5,
            secondary: 7
        }]
    );
    assert_eq!(
        host::read_on(&host, second),
        [Row {
            primary: 11,
            secondary: 7
        }]
    );
    let request = host.runtime().request(&principal, &scope).on_branch(second);
    let selected = request
        .corrections()
        .select(&source, history_limits())
        .unwrap();
    assert!(
        matches!(
            request
                .corrections()
                .reverse(&selected)
                .idempotency("shared-reverse")
                .limits(history_limits())
                .prepare()
                .unwrap(),
            WorthQueryApplicationCorrectionPreparation::Settled(
                WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
            )
        ),
        "the same client key on another target must resolve the original command and refuse drift"
    );
    assert_eq!(
        host::read_on(&host, second),
        [Row {
            primary: 11,
            secondary: 7
        }]
    );
    println!("inherited fork parent and cross-target command drift passed");
}

pub fn stale_prepared_target_refuses() {
    let host = host::install();
    let source = host::edit(&host, Value::Primary(11), "source");
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let selected = request
        .corrections()
        .select(&source, history_limits())
        .unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("stale-reverse")
        .limits(history_limits())
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("initial target must prepare");
    };
    host::edit(&host, Value::Secondary(19), "after-preparation");
    let outcome = prepared.publish().unwrap();
    assert!(
        matches!(&outcome, WorthQueryApplicationMutationOutcome::Commit(
        worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted::Denied(denial))
        if denial.kind() == worth_query_host::facade::primary_graph::WorthQueryApplicationCommitDenialKind::ProductBasisStale),
        "a prepared correction must preserve its exact target rather than readmitting: {outcome:?}"
    );
    assert_eq!(
        host::read(&host),
        [Row {
            primary: 11,
            secondary: 19
        }]
    );
    println!("stale prepared target refusal passed");
}

pub fn fresh_actor_can_reverse_after_original_actor_revocation() {
    use super::{principal_status::DisablePrincipal, query::Read};
    let host = host::install();
    let source = host::edit(&host, Value::Primary(11), "source");
    let scope = request_scope();
    let original_actor = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&original_actor, &scope);
    let selected = request
        .corrections()
        .select(&source, history_limits())
        .unwrap();
    let fresh_actor = authenticate_reviewer(host.installed_schema(), &scope);
    let fresh = host.runtime().request(&fresh_actor, &scope);
    let key = "disable-original-actor".to_owned();
    let disabled = fresh
        .advanced()
        .mutate(DisablePrincipal { identity: 1 })
        .without_source()
        .idempotency(&key)
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        disabled,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert!(
        matches!(
            request
                .corrections()
                .reverse(&selected)
                .idempotency("revoked-actor")
                .limits(history_limits())
                .prepare(),
            Err(WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::Principal(_)
            ))
        ),
        "a valid old authentication packet cannot bypass the current disabled principal mapping"
    );
    let selected = fresh
        .corrections()
        .select(&source, history_limits())
        .unwrap();
    let prepared = fresh
        .corrections()
        .reverse(&selected)
        .idempotency("fresh-actor")
        .limits(history_limits())
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("fresh actor must prepare");
    };
    let outcome = prepared.publish().unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "source semantic eligibility must exclude the original actor's revoked grants: {outcome:?}"
    );
    assert_eq!(
        fresh
            .query(Read {
                key: "numeric".to_owned()
            })
            .execute()
            .unwrap()
            .rows(),
        [Row {
            primary: 5,
            secondary: 7
        }]
    );
    println!("fresh inverse authority and original-grant independence passed");
}
