use super::*;

const fn span(space: u64, key: u64, first: u64, last: u64) -> ReachSpan {
    ReachSpan {
        space,
        key,
        first,
        last,
    }
}

#[test]
fn intent_admits_only_unreached_canonically_covered_unobligated_candidates() {
    let candidate = [span(1, 0, 0, 9), span(2, 7, 3, 3)];
    let canonical = ReclaimCoverage::CanonicalSuccessor;
    let plan = |reach: &[ReachSpan], obligations: &[ReachSpan], coverage| {
        plan_reclaim_intent(&candidate, reach, obligations, coverage)
    };
    // Disjoint by range, by key and by space.
    let unrelated = [span(1, 0, 10, 20), span(2, 8, 3, 3), span(3, 0, 0, 9)];
    assert_eq!(plan(&unrelated, &[], canonical), ReclaimVerdict::Admit);
    // M5: any protected key inside a candidate span refuses, at either edge.
    for reach in [span(1, 0, 9, 9), span(1, 0, 0, 0), span(2, 7, 0, 5)] {
        assert_eq!(plan(&[reach], &[], canonical), ReclaimVerdict::Reachable);
    }
    // M8: a derived accelerator alone is never sole truth.
    assert_eq!(
        plan(&unrelated, &[], ReclaimCoverage::DerivedOnly),
        ReclaimVerdict::Uncovered
    );
    assert_eq!(
        plan(&unrelated, &[], ReclaimCoverage::Uncovered),
        ReclaimVerdict::Uncovered
    );
    assert_eq!(
        plan(&unrelated, &[span(1, 0, 0, u64::MAX)], canonical),
        ReclaimVerdict::Obligated
    );
    // Inverted (empty) spans never overlap valid spans.
    assert!(!spans_overlap(span(1, 0, 10, 5), span(1, 0, 6, 9)));
    assert!(!spans_overlap(span(1, 0, 6, 9), span(1, 0, 10, 5)));
    // Reach outranks coverage and obligations.
    assert_eq!(
        plan(
            &[span(1, 0, 4, 4)],
            &[span(1, 0, 4, 4)],
            ReclaimCoverage::DerivedOnly
        ),
        ReclaimVerdict::Reachable
    );
}

#[test]
fn a_step_fits_only_within_the_remaining_budget_slice() {
    let budget = ReclaimBudget {
        max_artifacts: 3,
        max_bytes: 100,
    };
    assert!(budget_admits(2, 90, 10, budget));
    assert!(!budget_admits(3, 0, 0, budget));
    assert!(!budget_admits(0, 90, 11, budget));
    assert!(!budget_admits(0, 0, u64::MAX, budget));
    assert!(budget_admits(
        0,
        u64::MAX - 1,
        1,
        ReclaimBudget {
            max_artifacts: 1,
            max_bytes: u64::MAX
        }
    ));
}

#[test]
fn resume_never_releases_a_published_range_twice() {
    let fresh = resume_release(ReleaseBarrier::IntentDurable);
    assert!(fresh.publish_free_space && fresh.record_completion && fresh.admit_to_allocator);
    let interrupted = resume_release(ReleaseBarrier::FreeSpacePublished);
    assert!(!interrupted.publish_free_space && !interrupted.admit_to_allocator);
    assert!(interrupted.record_completion);
    let settled = resume_release(ReleaseBarrier::CompletionDurable);
    assert!(!settled.publish_free_space && !settled.record_completion);
    assert!(!settled.admit_to_allocator);
}
