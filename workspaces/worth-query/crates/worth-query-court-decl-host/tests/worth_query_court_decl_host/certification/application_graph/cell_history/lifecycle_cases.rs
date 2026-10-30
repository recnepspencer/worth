//! Old and current answers across actual cell replacement and deletion.
use super::{
    host,
    lifecycle::{ChangeCell, Denied, Replacement},
    live_at, values,
};
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;

#[test]
fn retained_cells_survive_replacement_and_retirement_without_restoring_the_head() {
    let host = host::install();
    let initial = live_at::current(&host, 1);
    let untouched = live_at::current(&host, 2);
    assert_eq!(values(&initial), [(10, 7), (11, 13)]);
    assert_eq!(values(&untouched), [(20, 29)]);
    let before = live_at::observe(&host);
    let replace = ChangeCell {
        feature: 1,
        cell: 10,
        replacement: Some(Replacement { key: 12, value: 47 }),
    };
    let replacement = live_at::change(&host, replace.clone(), 301);
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: replaced, ..
    } = replacement
    else {
        panic!("replacement commits: {replacement:?}")
    };
    let middle = live_at::current(&host, 1);
    assert_eq!(values(&middle), [(11, 13), (12, 47)]);
    assert_eq!(
        middle[0], initial[1],
        "the untouched cell keeps its occurrence"
    );
    assert_ne!(
        middle[1].occurrence, initial[0].occurrence,
        "replacement creates a new actual occurrence"
    );
    let middle_basis = live_at::observe(&host);
    assert!(matches!(
        live_at::change(
            &host,
            ChangeCell {
                feature: 1,
                cell: 11,
                replacement: None
            },
            302
        ),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let final_rows = live_at::current(&host, 1);
    assert_eq!(values(&final_rows), [(12, 47)]);
    assert_eq!(final_rows[0], middle[1]);
    let final_basis = live_at::observe(&host);
    assert_ne!(before.selected_commit(), middle_basis.selected_commit());
    assert_ne!(
        middle_basis.selected_commit(),
        final_basis.selected_commit()
    );
    assert_eq!(live_at::at(&host, 1, &before), initial);
    assert_eq!(live_at::at(&host, 1, &middle_basis), middle);
    assert_eq!(live_at::at_commit(&host, 1, &replaced), middle);
    assert_eq!(live_at::current(&host, 2), untouched);
    assert_eq!(live_at::at(&host, 2, &before), untouched);
    let WorthQueryApplicationMutationOutcome::AlreadyCommitted(retry) =
        live_at::change(&host, replace, 301)
    else {
        panic!("replacement retry rejoins its original commit")
    };
    assert!(retry.is_same_authoritative_commit(&replaced));
    assert_eq!(
        live_at::observe(&host).selected_commit(),
        final_basis.selected_commit()
    );
    assert_eq!(live_at::current(&host, 1), final_rows);
}

#[test]
fn structural_denials_preserve_both_features_and_a_lawful_replacement_commits() {
    let host = host::install();
    let first = live_at::current(&host, 1);
    let second = live_at::current(&host, 2);
    let before = live_at::observe(&host);
    for replacement in [None, Some(Replacement { key: 21, value: 53 })] {
        assert!(matches!(
            live_at::change(
                &host,
                ChangeCell {
                    feature: 1,
                    cell: 20,
                    replacement
                },
                401
            ),
            WorthQueryApplicationMutationOutcome::DomainDenied(Denied::CellNotOwned)
        ));
        assert_eq!(
            live_at::observe(&host).selected_commit(),
            before.selected_commit()
        );
        assert_eq!(live_at::current(&host, 1), first);
        assert_eq!(live_at::current(&host, 2), second);
    }
    assert!(matches!(
        live_at::change(
            &host,
            ChangeCell {
                feature: 2,
                cell: 20,
                replacement: Some(Replacement { key: 10, value: 53 })
            },
            402
        ),
        WorthQueryApplicationMutationOutcome::DomainDenied(Denied::ReplacementExists)
    ));
    assert_eq!(
        live_at::observe(&host).selected_commit(),
        before.selected_commit()
    );
    assert_eq!(live_at::current(&host, 1), first);
    assert_eq!(live_at::current(&host, 2), second);
    assert!(matches!(
        live_at::change(
            &host,
            ChangeCell {
                feature: 2,
                cell: 20,
                replacement: Some(Replacement { key: 21, value: 53 })
            },
            403
        ),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(values(&live_at::current(&host, 2)), [(21, 53)]);
    assert_ne!(
        live_at::current(&host, 2)[0].occurrence,
        second[0].occurrence
    );
    assert_eq!(live_at::current(&host, 1), first);
    assert_eq!(live_at::at(&host, 2, &before), second);
}
