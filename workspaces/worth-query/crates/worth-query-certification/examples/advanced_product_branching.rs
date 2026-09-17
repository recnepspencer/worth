//! Run with `cargo run -p worth-query-certification --example advanced_product_branching`.
//! Branch construction and bounded history inspection deliberately use the World API; all product
//! reads and mutations use the ordinary application entry.

pub mod product_workflow_support;

use std::num::NonZeroUsize;

use product_workflow_support::schema::AmendTemporalInput;
use product_workflow_support::{
    principal, read_input, AmendTemporalIntent, ExampleApplication, TemporalIntentRead,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
};

fn main() {
    std::thread::Builder::new()
        .name("advanced-product-branching".to_owned())
        .stack_size(8 * 1024 * 1024)
        .spawn(run)
        .expect("the bounded example thread must start")
        .join()
        .expect("the advanced product workflow must complete");
}

fn run() {
    let application = ExampleApplication::publish("ready");
    let scope = product_workflow_support::adapters::request_scope();
    let principal = principal(&application, &scope);
    let source = application.runtime.current_world();
    let source_value_before = read_input(&application, source, &principal, &scope);

    let reuse_both = application
        .runtime
        .branches()
        .fork(source)
        .components(|components| {
            components
                .reuse_exact_relational_basis()
                .reuse_exact_signal_basis()
        })
        .create()
        .expect("the exact reuse product must publish");
    let fork_relational = application
        .runtime
        .branches()
        .fork(source)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the Relational fork product must publish");
    let fork_signal = application
        .runtime
        .branches()
        .fork(source)
        .components(|components| components.reuse_exact_relational_basis().fork_signal())
        .create()
        .expect("the Signal fork product must publish");
    let fork_both = application
        .runtime
        .branches()
        .fork(source)
        .components(|components| components.fork_relational().fork_signal())
        .create()
        .expect("the fully independent product must publish");

    let first = mutate(
        &application,
        &principal,
        &scope,
        fork_relational,
        2,
        "branch-local-change",
        0x31,
    );
    assert_eq!(
        read_input(&application, fork_relational, &principal, &scope),
        "branch-local-change"
    );

    let newest = mutate(
        &application,
        &principal,
        &scope,
        fork_relational,
        3,
        "branch-newest",
        0x32,
    );
    assert_eq!(
        read_input(&application, fork_relational, &principal, &scope),
        "branch-newest"
    );

    let historical = application
        .runtime
        .request(&principal, &scope)
        .on_branch(fork_relational)
        .at_commit(&first, NonZeroUsize::new(2).unwrap())
        .expect("the first application receipt must select its retained generation")
        .query(TemporalIntentRead {
            identity: "intent-1".to_owned(),
        })
        .execute()
        .expect("the historical read must use the retained application evidence");
    assert_eq!(historical.rows()[0].input, "branch-local-change");

    let history = application
        .runtime
        .branches()
        .history(fork_relational, NonZeroUsize::new(1).unwrap())
        .expect("the live product must expose bounded ancestry");
    assert_eq!(history.visited_count(), 1);
    assert!(!history.is_complete());
    assert!(history.next_parent_commit().is_some());

    assert_eq!(
        read_input(&application, source, &principal, &scope),
        source_value_before
    );
    drop(historical);
    drop(first);
    drop(history);
    drop(newest);
    for branch in [reuse_both, fork_signal, fork_both, fork_relational] {
        let closed = application
            .runtime
            .on_branch(branch)
            .close()
            .expect("released branch resources must close");
        assert!(closed.is_complete());
    }
    assert!(application.runtime.branches().pending_cleanup().is_empty());
    println!("created, advanced, inspected, and closed all product postures through the application entry");
}

fn mutate(
    application: &ExampleApplication,
    principal: &worth_query_host::facade::admission::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<
        product_workflow_support::schema::TemporalHostSchema,
    >,
    scope: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    revision: u64,
    input: &str,
    idempotency: u64,
) -> worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt {
    let outcome = application
        .runtime
        .request(principal, scope)
        .on_branch(branch)
        .mutate(AmendTemporalIntent {
            identity: "intent-1".to_owned(),
            amendment: AmendTemporalInput {
                revision,
                due: 11,
                lifecycle: "active".to_owned(),
                input: input.to_owned(),
                gate: "ready".to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .execute_in_program(&application.runtime)
        .expect("the branch-local application mutation must prepare");
    let WorthQueryApplicationMutationOutcome::Committed { receipt, result } = outcome else {
        panic!("the branch-local application mutation must commit")
    };
    assert_eq!(result.revision, revision);
    receipt
}
