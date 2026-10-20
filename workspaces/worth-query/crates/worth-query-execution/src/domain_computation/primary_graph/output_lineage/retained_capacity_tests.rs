//! Original backing owns one ledger grant through its final shared holder.
use super::*;

#[test]
fn ordinary_lineage_retains_two_large_originals_and_refunds_only_final_holders() {
    let ledger = LineageRetentionLedger::default();
    let bytes = 40 * 1024 * 1024;
    let first = ledger.reserve(bytes).unwrap();
    let mut backing = Vec::<u8>::new();
    backing.try_reserve_exact(bytes as usize).unwrap();
    backing.resize(bytes as usize, 7);
    let original = Arc::new((backing, first));
    let clone = Arc::clone(&original);
    assert_eq!(ledger.retained_bytes(), bytes);
    let second = ledger.reserve(bytes);
    println!(
        "lineage admission: requested={bytes}, live={}, maximum={:?}, accepted={}",
        ledger.retained_bytes(),
        ledger.state.lock().unwrap().maximum_bytes,
        second.is_ok()
    );
    let accepted = second.is_ok();
    let second = second.ok().map(|ticket| {
        let mut backing = Vec::<u8>::new();
        backing.try_reserve_exact(bytes as usize).unwrap();
        backing.resize(bytes as usize, 9);
        Arc::new((backing, ticket))
    });
    drop(original);
    assert_eq!(clone.0[0], 7);
    assert_eq!(
        ledger.retained_bytes(),
        if accepted { 2 * bytes } else { bytes }
    );
    drop(clone);
    if let Some(second) = second {
        let clone = Arc::clone(&second);
        drop(second);
        assert_eq!(ledger.retained_bytes(), bytes);
        assert_eq!(clone.0[0], 9);
        drop(clone);
    }
    assert_eq!(
        ledger.retained_bytes(),
        0,
        "all real backing grants released"
    );
    assert!(
        accepted,
        "ordinary profile refuses the second original at an arbitrary quota"
    );
}

#[test]
fn finite_lineage_growth_refuses_without_releasing_prior_and_refunds_final_drop() {
    let ledger = LineageRetentionLedger::new(Some(19));
    let mut prior = ledger.reserve(11).unwrap();
    assert!(prior.reserve_additional(9).is_err());
    assert_eq!(prior.bytes(), 11);
    assert_eq!(ledger.retained_bytes(), 11);
    prior.reserve_additional(8).unwrap();
    assert_eq!(ledger.retained_bytes(), 19);
    assert!(ledger.reserve(1).is_err());
    prior.release_part(3);
    let next = ledger.reserve(3).unwrap();
    drop(prior);
    assert_eq!(ledger.retained_bytes(), 3);
    drop(next);
    assert_eq!(ledger.retained_bytes(), 0);
}

#[test]
fn absent_policy_still_rejects_accounting_overflow_without_releasing_prior() {
    let ledger = LineageRetentionLedger::new(None);
    let mut prior = ledger.reserve(11).unwrap();
    assert!(ledger.reserve(u64::MAX).is_err());
    assert!(prior.reserve_additional(u64::MAX).is_err());
    assert_eq!(prior.bytes(), 11);
    assert_eq!(ledger.retained_bytes(), 11);
    drop(prior);
    assert_eq!(ledger.retained_bytes(), 0);
}
