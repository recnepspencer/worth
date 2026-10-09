use worth_store_recovery_physics::{
    decide_ordered_root_step_basis, CheckpointRetiredReleaseIntent, OrderedRootStepBasis,
    RetirementReleaseIntent,
};
use worth_store_wal::{LogSequenceNumber, WalLsnRange};

use super::check_basis;
use crate::physical_runtime::recovery_construction::selected_rejoin::SelectedMediaRejoinDenial as Denial;

const CUTOFF: u64 = 16;
const SOURCE: u64 = 14;

fn intent() -> RetirementReleaseIntent {
    let lsn = WalLsnRange::new(LogSequenceNumber::new(15), LogSequenceNumber::new(16)).unwrap();
    RetirementReleaseIntent::new(lsn, SOURCE, SOURCE + 1, [7; 32])
}

fn planned() -> CheckpointRetiredReleaseIntent {
    match decide_ordered_root_step_basis(SOURCE, true, CUTOFF, false, &[intent()]) {
        Ok(OrderedRootStepBasis::RetirementIntent(basis)) => basis,
        other => panic!("expected a retirement basis, got {other:?}"),
    }
}

#[test]
fn a_retirement_edge_needs_store_to_find_no_member_leaving_its_source() {
    let intents = [intent()];
    let check = |member_leaves: bool| {
        check_basis(Some(planned()), true, SOURCE, CUTOFF, &intents, || {
            Ok(member_leaves)
        })
    };
    assert!(check(false).is_ok());
    // Store's own reread found a member: the plan's retirement kind is wrong.
    assert!(matches!(check(true), Err(Denial::WalFate)));
    let unreadable = check_basis(Some(planned()), true, SOURCE, CUTOFF, &intents, || {
        Err(Denial::BoundExceeded)
    });
    assert!(matches!(unreadable, Err(Denial::BoundExceeded)));
}

#[test]
fn a_member_edge_is_never_replaced_by_a_covered_intent() {
    let read_members = || -> Result<bool, Denial> { panic!("member edges match their own member") };
    assert!(check_basis(None, true, SOURCE, CUTOFF, &[], read_members).is_ok());
    assert!(matches!(
        check_basis(None, true, SOURCE, CUTOFF, &[intent()], read_members),
        Err(Denial::WalFate)
    ));
    // After a member edge, a covered intent no longer authorizes an edge.
    let no_member = || Ok(false);
    let after_member = check_basis(
        Some(planned()),
        false,
        SOURCE,
        CUTOFF,
        &[intent()],
        no_member,
    );
    assert!(matches!(after_member, Err(Denial::WalFate)));
}
