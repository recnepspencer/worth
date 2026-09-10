//! Run with `cargo run -p worth-query-certification --example advanced_product_branching`.

pub mod product_workflow_support;

use std::num::NonZeroUsize;

use product_workflow_support::{principal, read_input, read_selected, ExampleApplication};

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
    let request = product_workflow_support::adapters::request_scope();
    let principal = principal(&application, &request);
    let source = application.runtime.current_world();
    let source_commit_before = application
        .runtime
        .on_branch(source)
        .select()
        .expect("the source product must be selectable")
        .product()
        .selected_commit()
        .clone();
    let source_value_before = read_input(&application, source, &principal, &request);

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

    for branch in [reuse_both, fork_relational, fork_signal, fork_both] {
        let selected = application
            .runtime
            .on_branch(branch)
            .select()
            .expect("each exact product posture must remain selectable");
        assert_eq!(selected.product().product_branch(), branch);
    }

    let retained = application
        .runtime
        .on_branch(fork_relational)
        .select()
        .expect("the independent Relational branch must be retained");
    let retained_commit = retained.product().selected_commit().clone();
    let admitted = application.admit_input_change(
        application
            .runtime
            .on_branch(fork_relational)
            .select()
            .expect("the branch-local mutation must select its exact product"),
        "branch-local-change",
        0x31,
    );
    let committed = application
        .runtime
        .on_branch(fork_relational)
        .transaction()
        .apply(admitted)
        .commit()
        .expect("the transaction must retain the admitted product")
        .require_committed()
        .expect("the selected branch-local operation must commit");
    assert_eq!(committed.product_branch(), fork_relational);
    let branch_after = read_input(&application, fork_relational, &principal, &request);
    assert_eq!(branch_after, "branch-local-change");
    let branch_commit_after = application
        .runtime
        .on_branch(fork_relational)
        .select()
        .expect("the advanced branch must remain selectable")
        .product()
        .selected_commit()
        .clone();
    assert_ne!(branch_commit_after, retained_commit);

    let admitted = application.admit_input_change(
        application
            .runtime
            .on_branch(fork_relational)
            .select()
            .expect("the advanced branch must select its latest product"),
        "branch-newest",
        0x32,
    );
    let newest = application
        .runtime
        .on_branch(fork_relational)
        .transaction()
        .apply(admitted)
        .commit()
        .expect("the second transaction must retain the admitted product")
        .require_committed()
        .expect("the second branch-local operation must commit");
    assert_eq!(
        read_input(&application, fork_relational, &principal, &request),
        "branch-newest"
    );

    let first_page = application
        .runtime
        .branches()
        .history(fork_relational, NonZeroUsize::new(1).unwrap())
        .expect("the live product must expose bounded ancestry");
    assert_eq!(first_page.visited_count(), 1);
    assert!(!first_page.is_complete());
    assert!(first_page.next_parent_commit().is_some());
    let prior_page = first_page
        .continue_ancestry(NonZeroUsize::new(1).unwrap())
        .expect("the owner-issued continuation must advance one bounded page");
    let prior_entry = prior_page
        .entries()
        .next()
        .expect("the prior branch generation must remain retained");
    assert_eq!(prior_entry.selected_commit(), &branch_commit_after);
    let historical = prior_page
        .select(&prior_entry)
        .expect("an entry can select only its exact retained product basis");
    assert_eq!(
        read_selected(&application, historical, &principal, &request),
        "branch-local-change"
    );
    drop(prior_page);
    drop(first_page);

    let recovery_page = application
        .runtime
        .branches()
        .recovery_page(None, NonZeroUsize::new(1).unwrap())
        .expect("bounded recovery inspection must remain available");
    assert!(recovery_page.examined() <= 1);
    assert!(recovery_page.rows().is_empty());

    let source_value_after = read_input(&application, source, &principal, &request);
    let source_commit_after = application
        .runtime
        .on_branch(source)
        .select()
        .expect("the source product must remain selectable")
        .product()
        .selected_commit()
        .clone();
    assert_eq!(source_value_after, source_value_before);
    assert_eq!(source_commit_after, source_commit_before);

    drop(committed);
    drop(newest);
    drop(retained);
    for branch in [reuse_both, fork_signal, fork_both, fork_relational] {
        let closed = application
            .runtime
            .on_branch(branch)
            .close()
            .expect("released branch resources must close");
        assert!(closed.is_complete());
    }
    assert!(application.runtime.branches().pending_cleanup().is_empty());
    println!("created, advanced, inspected, and closed all product postures");
}
