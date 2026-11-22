//! Journey 4: `where_equal` selection plus residual filter, ordering and limits.
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial as Denial;
use worth_query_host::facade::primary_graph::WorthQueryApplicationOneShotDenialKind as Kind;

fn open(min_rank: i64) -> ByStatus {
    ByStatus {
        project: 1,
        status: "open".into(),
        min_rank,
    }
}

#[test]
fn status_equality_and_rank_floor_compose_with_deterministic_ordering() {
    let host = install(&board());
    // Open tasks 1(5) 2(9) 4(2) 5(9); rank >= 5 keeps 1, 2, 5; rank
    // descending then key ascending orders the tie between 2 and 5.
    assert_eq!(keys(world::by_status(&host, open(5), None)), [2, 5, 1]);
    assert_eq!(keys(world::by_status(&host, open(9), None)), [2, 5]);
    let done = ByStatus {
        project: 1,
        status: "done".into(),
        min_rank: 0,
    };
    assert_eq!(keys(world::by_status(&host, done, None)), [3]);
}

#[test]
fn the_result_limit_applies_to_filtered_rows_not_index_candidates() {
    let host = install(&board());
    // Four open candidates are selected; three survive the filter.
    let three = NonZeroUsize::new(3).unwrap();
    assert_eq!(
        keys(world::by_status(&host, open(5), Some(three))),
        [2, 5, 1]
    );
    let two = NonZeroUsize::new(2).unwrap();
    match world::by_status(&host, open(5), Some(two)) {
        Err(Denial::Execution(denial)) => assert_eq!(denial.kind(), Kind::ResultLimitExceeded),
        other => panic!("three filtered rows exceed a limit of two: {other:?}"),
    }
}
