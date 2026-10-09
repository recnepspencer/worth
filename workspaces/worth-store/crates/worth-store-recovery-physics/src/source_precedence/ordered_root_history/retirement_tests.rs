use worth_store_wal::{LogSequenceNumber, WalLsnRange};

use super::{
    decide_ordered_root_step_basis as decide, OrderedRootStepBasis, RetirementReleaseIntent,
};
use crate::OrderedRootHistoryDenial as Denial;

const CUTOFF: u64 = 16;
const SOURCE: u64 = 14;

fn intent(start: u64, candidate: u64, digest: u8) -> RetirementReleaseIntent {
    let lsn = WalLsnRange::new(
        LogSequenceNumber::new(start),
        LogSequenceNumber::new(start + 1),
    )
    .unwrap();
    RetirementReleaseIntent::new(lsn, SOURCE, candidate, [digest; 32])
}

fn admitted(result: Result<OrderedRootStepBasis, Denial>) -> RetirementReleaseIntent {
    match result {
        Ok(OrderedRootStepBasis::RetirementIntent(basis)) => {
            assert_eq!(basis.checkpoint_cutoff(), CUTOFF);
            basis.intent()
        }
        other => panic!("expected a retirement basis, got {other:?}"),
    }
}

#[test]
fn covered_intent_authorizes_a_prefix_edge_without_a_member() {
    let covered = intent(15, SOURCE + 1, 7);
    assert_eq!(
        admitted(decide(SOURCE, true, CUTOFF, false, &[covered])),
        covered
    );
}

#[test]
fn intent_above_the_cutoff_never_authorizes_an_edge() {
    // The release WAL is still live above the cutoff: no member stays a denial.
    let live = intent(CUTOFF, SOURCE + 1, 7);
    assert_eq!(
        decide(SOURCE, true, CUTOFF, false, &[live]),
        Err(Denial::Source)
    );
    // A live intent beside a member neither authorizes nor blocks the member.
    assert_eq!(
        decide(SOURCE, true, CUTOFF, true, &[live]),
        Ok(OrderedRootStepBasis::WalMember)
    );
}

#[test]
fn member_and_covered_intent_for_one_source_are_ambiguous() {
    let covered = intent(15, SOURCE + 1, 7);
    assert_eq!(
        decide(SOURCE, true, CUTOFF, true, &[covered]),
        Err(Denial::Effect)
    );
    assert_eq!(
        decide(SOURCE, true, CUTOFF, true, &[]),
        Ok(OrderedRootStepBasis::WalMember)
    );
}

#[test]
fn covered_intent_authorizes_only_a_retirement_prefix_edge_to_its_successor() {
    let covered = intent(15, SOURCE + 1, 7);
    assert_eq!(
        decide(SOURCE, false, CUTOFF, false, &[covered]),
        Err(Denial::Source)
    );
    assert_eq!(
        decide(SOURCE + 1, true, CUTOFF, false, &[covered]),
        Err(Denial::Source)
    );
    let skipping = intent(15, SOURCE + 2, 7);
    assert_eq!(
        decide(SOURCE, true, CUTOFF, false, &[skipping]),
        Err(Denial::Source)
    );
}

#[test]
fn repeated_intents_must_name_one_release() {
    let first = intent(14, SOURCE + 1, 7);
    let resynchronized = intent(15, SOURCE + 1, 7);
    assert_eq!(
        admitted(decide(
            SOURCE,
            true,
            CUTOFF,
            false,
            &[first, resynchronized]
        )),
        first
    );
    let conflicting = intent(15, SOURCE + 1, 8);
    assert_eq!(
        decide(SOURCE, true, CUTOFF, false, &[first, conflicting]),
        Err(Denial::Source)
    );
}
