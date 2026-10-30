//! Public cell-bearing publications and genuine Native historical reads.
//! Functional qualification only; scale and total retained bytes are separate evidence.
#[path = "cell_history/cost.rs"]
mod cost;
#[path = "cell_history/edit.rs"]
mod edit;
#[path = "cell_history/host.rs"]
mod host;
#[path = "cell_history/live_at.rs"]
mod live_at;
#[path = "cell_history/query.rs"]
mod query;
#[path = "cell_history/schema.rs"]
mod schema;

use query::CellRow;
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;

fn values(rows: &[CellRow]) -> Vec<(u64, u64)> {
    rows.iter().map(|row| (row.key, row.value)).collect()
}

#[test]
fn retained_feature_steps_read_original_cell_versions_after_later_local_edits() {
    let host = host::install();
    let initial = live_at::current(&host, 1);
    assert_eq!(values(&initial), [(10, 7), (11, 13)]);
    assert_eq!(values(&live_at::current(&host, 2)), [(20, 29)]);
    let step_zero = live_at::observe(&host);
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: step_one, ..
    } = live_at::edit(&host, 1, 10, 31, 101)
    else {
        panic!("the first cell edit commits")
    };
    let step_one_read = live_at::observe(&host);
    assert_ne!(step_zero.selected_commit(), step_one_read.selected_commit());
    let WorthQueryApplicationMutationOutcome::Committed { .. } =
        live_at::edit(&host, 1, 10, 43, 102)
    else {
        panic!("the later cell edit commits")
    };
    let current = live_at::current(&host, 1);
    let current_basis = live_at::observe(&host);
    assert_ne!(
        step_one_read.selected_commit(),
        current_basis.selected_commit()
    );
    assert_eq!(values(&current), [(10, 43), (11, 13)]);
    assert_eq!(live_at::at(&host, 1, &step_zero), initial);
    assert_eq!(
        values(&live_at::at(&host, 1, &step_one_read)),
        [(10, 31), (11, 13)]
    );
    assert_eq!(
        live_at::at_commit(&host, 1, &step_one),
        live_at::at(&host, 1, &step_one_read)
    );
    assert_eq!(
        initial.iter().map(|row| row.occurrence).collect::<Vec<_>>(),
        current.iter().map(|row| row.occurrence).collect::<Vec<_>>(),
        "Native versions preserve the actual cell occurrences"
    );
    assert_eq!(
        values(&live_at::current(&host, 2)),
        [(20, 29)],
        "another feature remains unchanged"
    );
    let WorthQueryApplicationMutationOutcome::AlreadyCommitted(replayed) =
        live_at::edit(&host, 1, 10, 31, 101)
    else {
        panic!("the original key rejoins its commit")
    };
    assert!(
        replayed.is_same_authoritative_commit(&step_one),
        "the retry must identify the original authoritative commit"
    );
    assert_eq!(
        live_at::observe(&host).selected_commit(),
        current_basis.selected_commit(),
        "recovered retry leaves the actual current head unchanged"
    );
    assert_eq!(
        values(&live_at::current(&host, 1)),
        [(10, 43), (11, 13)],
        "retry must not republish an earlier cell value"
    );
}

#[test]
fn a_feature_cannot_edit_a_cell_owned_by_another_feature() {
    let host = host::install();
    let before = live_at::observe(&host);
    let outcome = live_at::edit(&host, 1, 20, 97, 201);
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::DomainDenied(edit::Denied::CellNotOwned)
    ));
    assert_eq!(
        live_at::observe(&host).selected_commit(),
        before.selected_commit(),
        "foreign-feature denial has no publication effect"
    );
    assert_eq!(values(&live_at::current(&host, 1)), [(10, 7), (11, 13)]);
    assert_eq!(values(&live_at::current(&host, 2)), [(20, 29)]);
    assert_eq!(live_at::current(&host, 2), live_at::at(&host, 2, &before));
    assert!(matches!(
        live_at::edit(&host, 2, 20, 97, 202),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(values(&live_at::current(&host, 2)), [(20, 97)]);
}
