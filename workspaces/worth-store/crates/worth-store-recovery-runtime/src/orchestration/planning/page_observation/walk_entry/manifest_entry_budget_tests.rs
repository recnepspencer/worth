use std::num::NonZeroUsize;

use super::{
    manifest_entry_limit_for_test, pays_for, spend, ChargeTarget, EntriesStopped, EntryAdmission,
    ManifestEntryBudget, ViewEntryCap, ROOT_ENTRY,
};

fn past(observed: u64, admitted: u64) -> EntriesStopped {
    EntriesStopped::Limit(manifest_entry_limit_for_test(observed, admitted))
}

const ROOT: ChargeTarget = ChargeTarget::root(7);
const THREE: NonZeroUsize = NonZeroUsize::MIN.saturating_add(2);

#[test]
fn a_refused_charge_names_the_count_it_would_have_reached() {
    let mut budget = ManifestEntryBudget::for_test(10, 7);
    assert!(budget.charge(THREE, ROOT).is_ok());
    assert_eq!(budget.refused(), None);
    assert_eq!(budget.charge(ROOT_ENTRY, ROOT).err(), Some(past(11, 10)));
    assert_eq!(
        budget.refused().map(EntriesStopped::Limit),
        Some(past(11, 10))
    );

    let mut budget = ManifestEntryBudget::for_test(10, 7);
    assert_eq!(budget.admit(5), Err(past(12, 10)));
    // Handed the 3 left, the decoder counted 4 of its own.
    assert_eq!(budget.refuse_decoded(4), past(11, 10));
    assert_eq!(
        budget.refused().map(EntriesStopped::Limit),
        Some(past(11, 10))
    );
    // Handed all 10, one view held 13.
    let view = ViewEntryCap::of(&budget).refuse(13);
    assert_eq!(budget.view_refused(view), past(13, 10));
    assert_eq!(
        budget.refused().map(EntriesStopped::Limit),
        Some(past(13, 10))
    );
}

#[test]
fn a_count_past_every_count_is_no_limit() {
    let mut budget = ManifestEntryBudget::for_test(u64::MAX - 1, u64::MAX - 2);
    assert_eq!(budget.admit(2), Err(past(u64::MAX, u64::MAX - 1)));
    // An overflow leaves no earlier refusal standing for it.
    assert_eq!(budget.admit(3), Err(EntriesStopped::CountOverflow));
    assert_eq!(budget.refused(), None);
    assert_eq!(
        budget.refuse_decoded(u64::MAX),
        EntriesStopped::CountOverflow
    );
    assert_eq!(budget.refused(), None);
}

#[test]
fn a_charge_mints_a_token_for_its_root_only_within_the_budget() {
    let mut budget = ManifestEntryBudget::for_test(10, 8);
    let token = budget.charge(ROOT_ENTRY, ROOT).expect("one entry is left");
    assert_eq!(*token.outcome(), ROOT);
    spend(token, 7);
    assert_eq!(budget.remaining(), 1);
    let next = ChargeTarget::root(9);
    let token = budget.charge(ROOT_ENTRY, next).expect("the last entry");
    pays_for(&token, 9);
    assert_eq!(token.outcome().generation(), 9);
    spend(token, 9);
    assert_eq!(budget.remaining(), 0);
    assert_eq!(budget.charge(ROOT_ENTRY, ROOT).err(), Some(past(11, 10)));
}

#[test]
fn a_view_refuses_past_its_cap_and_charges_its_budget_nothing() {
    let budget = ManifestEntryBudget::for_test(10, 9);
    let mut view = ViewEntryCap::of(&budget);
    assert_eq!(view.remaining(), 10);
    assert_eq!(view.admit(10), Ok(()));
    assert_eq!(view.admit(1), Err(past(11, 10)));
    assert_eq!(view.refuse_decoded(2), past(12, 10));
    assert_eq!(view.refuse(10), EntriesStopped::CountOverflow);
    assert_eq!(budget.remaining(), 1);
    assert_eq!(budget.refused(), None);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "a charge pays for the read of its own target")]
fn a_charge_for_one_root_handed_to_a_read_of_another_trips_the_check() {
    let mut budget = ManifestEntryBudget::for_test(10, 0);
    let token = budget.charge(ROOT_ENTRY, ChargeTarget::root(8));
    spend(token.expect("ample"), 7);
}
