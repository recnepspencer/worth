//! A transaction forks the current head, steps on the fork, and adopts it by
//! closing the head it replaces. The adopted fork keeps its parent's history
//! as its own, so corrections reach across the adoption, and retained history
//! grows by the same amount each cycle.
use super::{
    authentication::*,
    edit::{Edited, MissingValue, Value},
    host,
    other_edit::OtherEdit,
    query::Row,
    tree_reapplication::*,
};
use worth_query_host::facade::{
    application_entry::*, application_installation::WorthQueryCheckpointCapturePolicy,
    primary_graph::*, product::WorthQueryProductBranch, runtime::ExecutionAllocationPolicy,
};

pub fn run() {
    adoption_cycles_retain_linear_history();
    refused_fork_leaves_its_parent_unchanged();
    corrections_reach_across_an_adoption();
    super::fork_head_cleanup::run();
}

const CYCLES: usize = 100;

fn adoption_cycles_retain_linear_history() {
    let host = host::install();
    let mut head = fork(&host, host.runtime().current_world());
    let anchor = host::edit_on(&host, head, Value::Primary(11), "anchor");
    let mut bytes = vec![checkpoint_bytes(&host)];
    let mut steady_evidence = 0;
    for cycle in 1..=CYCLES {
        let next = fork(&host, head);
        let value = 20 + (cycle % 2) as u64;
        host::edit_on(
            &host,
            next,
            Value::Secondary(value),
            &format!("step-{cycle:03}"),
        );
        adopt(&host, head);
        head = next;
        assert!(
            host.runtime().branches().pending_cleanup().is_empty(),
            "cycle {cycle}: the replaced head's cleanup completes"
        );
        let evidence = host.runtime().completed_evidence_retained_for_test();
        if cycle == 2 {
            steady_evidence = evidence;
        }
        assert!(
            cycle < 2 || evidence == steady_evidence,
            "cycle {cycle}: a replaced head's evidence goes once history lets its commits go"
        );
        if cycle % (CYCLES / 2) == 0 {
            bytes.push(checkpoint_bytes(&host));
        }
    }
    let first = bytes[1] - bytes[0];
    let second = bytes[2] - bytes[1];
    println!(
        "fork-head adoption retained bytes: start {}, half {first}, second half {second}",
        bytes[0]
    );
    // Replaying a journal per fork grows each half by more than the last;
    // linear retention grows both halves alike.
    assert!(
        second <= first + first / 10,
        "retained history grows linearly in adoptions: {first} then {second}"
    );
    assert_eq!(
        host::read_on(&host, head),
        [Row {
            primary: 11,
            secondary: 20
        }]
    );
    reverse_on(&host, head, &anchor, "anchor-reverse");
    assert_eq!(
        host::read_on(&host, head),
        [Row {
            primary: 5,
            secondary: 20
        }],
        "a source a hundred adoptions back stays correctable"
    );
    println!("fork-head adoption cycles kept linear history and their oldest correction");
}

fn refused_fork_leaves_its_parent_unchanged() {
    let host = host::install();
    let parent = fork(&host, host.runtime().current_world());
    let source = host::edit_on(&host, parent, Value::Primary(11), "parent-source");
    let before = selected_commit(&host, parent);
    let refused = fork(&host, parent);
    host::edit_on(&host, refused, Value::Secondary(19), "fork-step");
    assert!(
        refuses(&host, refused, Value::Primary(500), "fork-refused"),
        "a value above the ceiling refuses the transaction"
    );
    adopt(&host, refused);
    assert_eq!(selected_commit(&host, parent), before);
    assert_eq!(
        host::read_on(&host, parent),
        [Row {
            primary: 11,
            secondary: 7
        }]
    );
    reverse_on(&host, parent, &source, "parent-reverse");
    assert_eq!(
        host::read_on(&host, parent),
        [Row {
            primary: 5,
            secondary: 7
        }]
    );
    assert!(host.runtime().branches().pending_cleanup().is_empty());
    println!("a refused fork closes and leaves its parent's head and history unchanged");
}

fn corrections_reach_across_an_adoption() {
    for fork_made in [false, true] {
        let host = host::install();
        let parent = fork(&host, host.runtime().current_world());
        let parent_source = host::edit_on(&host, parent, Value::Primary(11), "parent-made");
        let adopted = fork(&host, parent);
        let source = if fork_made {
            host::edit_on(&host, adopted, Value::Primary(13), "fork-made")
        } else {
            host::edit_on(&host, adopted, Value::Secondary(19), "fork-step");
            parent_source
        };
        adopt(&host, parent);
        let reversal = reverse_on(&host, adopted, &source, "across-adoption");
        let restored = if fork_made { 11 } else { 5 };
        let secondary = if fork_made { 7 } else { 19 };
        assert_eq!(
            host::read_on(&host, adopted),
            [Row {
                primary: restored,
                secondary
            }]
        );
        assert_eq!(
            reversal.commit_reference().branch_id,
            host::edit_on(&host, adopted, Value::Primary(17), "after")
                .commit_reference()
                .branch_id,
            "the correction lands on the adopted fork"
        );
    }
    println!("corrections reverse parent-made and fork-made sources across an adoption");
}

pub(super) fn fork(host: &host::Host, from: WorthQueryProductBranch) -> WorthQueryProductBranch {
    host.with_host_advancement(|phase| {
        host.runtime()
            .branches()
            .fork(from)
            .components(|components| components.fork_relational().reuse_exact_signal_basis())
            .create(&phase)
            .unwrap()
    })
    .unwrap()
}

/// Closes a replaced head; its cleanup must finish now, not wait on its fork.
pub(super) fn adopt(host: &host::Host, replaced: WorthQueryProductBranch) {
    let closed = host
        .with_host_advancement(|phase| {
            host.runtime()
                .on_branch(replaced)
                .close(&phase)
                .unwrap_or_else(|denial| panic!("a replaced head closes: {denial:?}"))
        })
        .unwrap();
    assert!(closed.is_complete());
}

fn checkpoint_bytes(host: &host::Host) -> usize {
    host.with_host_advancement(|phase| {
        host.runtime()
            .capture_application_checkpoint(
                &phase,
                WorthQueryCheckpointCapturePolicy::SystemAllocation,
            )
            .unwrap()
    })
    .unwrap()
    .bytes()
    .len()
}

fn selected_commit(
    host: &host::Host,
    branch: WorthQueryProductBranch,
) -> impl Eq + std::fmt::Debug {
    host.with_host_advancement(|phase| {
        host.runtime()
            .on_branch(branch)
            .select(&phase)
            .unwrap()
            .product()
            .selected_commit()
            .clone()
    })
    .unwrap()
}

/// Whether the ordinary edit refuses in the domain, publishing nothing.
fn refuses(host: &host::Host, branch: WorthQueryProductBranch, value: Value, key: &str) -> bool {
    matches!(
        mutate(host, branch, value, key),
        WorthQueryApplicationMutationOutcome::DomainDenied(_)
    )
}

/// One admitted ordinary edit on `branch`, whatever its outcome.
pub(super) fn mutate(
    host: &host::Host,
    branch: WorthQueryProductBranch,
    value: Value,
    key: &str,
) -> WorthQueryApplicationMutationOutcome<MissingValue, Edited> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .advanced()
        .mutate(OtherEdit {
            key: "numeric".to_owned(),
            value,
        })
        .without_source()
        .idempotency(&key.to_owned())
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .expect("the ordinary edit is admitted")
}

fn reverse_on(
    host: &host::Host,
    branch: WorthQueryProductBranch,
    source: &WorthQueryApplicationCommitReceipt,
    key: &str,
) -> WorthQueryApplicationCommitReceipt {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope).on_branch(branch);
    let selected = request.corrections().select(source, limits()).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency(key)
        .limits(limits())
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("a fresh reversal prepares")
    };
    match prepared.publish().unwrap() {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => receipt,
        other => panic!("a fresh reversal commits: {other:?}"),
    }
}
