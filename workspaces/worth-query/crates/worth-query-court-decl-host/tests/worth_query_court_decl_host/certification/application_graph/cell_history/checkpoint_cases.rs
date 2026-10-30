//! Reconstruction consumes only bytes after all source runtime custody is gone.
use super::{
    checkpoint, host,
    lifecycle::{ChangeCell, Replacement},
    live_at, values,
};
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;

#[test]
fn cell_checkpoint_restores_structural_edits_and_retries_by_the_durable_record() {
    let replace = ChangeCell {
        feature: 1,
        cell: 10,
        replacement: Some(Replacement { key: 12, value: 47 }),
    };
    let (bytes, survivor, untouched, original_commit) = {
        let host = host::install();
        assert!(matches!(
            live_at::edit(&host, 1, 10, 31, 501),
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        let replacement = live_at::change(&host, replace.clone(), 502);
        let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = replacement else {
            panic!("the replacement commits: {replacement:?}")
        };
        let original_commit = receipt.committed_changes().commit_reference().commit_id;
        assert!(matches!(
            live_at::change(
                &host,
                ChangeCell {
                    feature: 1,
                    cell: 11,
                    replacement: None
                },
                503
            ),
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        let current = live_at::current(&host, 1);
        let untouched = live_at::current(&host, 2);
        assert_eq!(values(&current), [(12, 47)]);
        assert_eq!(values(&untouched), [(20, 29)]);
        // Rows and commit identity are weak expected data, not runtime custody.
        (
            checkpoint::capture(&host),
            current[0].clone(),
            untouched,
            original_commit,
        )
    };
    let restored = checkpoint::restore(bytes, 64);
    assert_eq!(live_at::current(&restored, 1), [survivor.clone()]);
    assert_eq!(live_at::current(&restored, 2), untouched);
    let old = live_at::observe(&restored);
    assert!(matches!(
        live_at::edit(&restored, 1, 12, 61, 504),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(values(&live_at::current(&restored, 1)), [(12, 61)]);
    assert_eq!(
        live_at::current(&restored, 1)[0].occurrence,
        survivor.occurrence
    );
    assert_eq!(live_at::at(&restored, 1, &old), [survivor]);
    assert_eq!(live_at::current(&restored, 2), untouched);
    let head = live_at::observe(&restored);
    let retry = live_at::change(&restored, replace, 502);
    let WorthQueryApplicationMutationOutcome::PreviouslyCommitted(record) = &retry else {
        panic!("restoration resolves the durable record: {retry:?}")
    };
    assert_eq!(record.commit_id(), original_commit);
    assert!(retry.receipt().is_none(), "history grants no live receipt");
    assert!(
        retry.result().is_none(),
        "history invents no handler result"
    );
    assert_eq!(
        live_at::observe(&restored).selected_commit(),
        head.selected_commit()
    );
    assert_eq!(values(&live_at::current(&restored, 1)), [(12, 61)]);
    assert_eq!(live_at::current(&restored, 2), untouched);
}
