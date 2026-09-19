use super::{BridgeRetentionDenial as D, BridgeRetentionLedger};
use crate::policy::BridgeConditionalRetentionBudget;

#[test]
fn aggregate_count_and_byte_reservations_are_checked_and_return_on_last_owner() {
    let total = 256;
    let budget = BridgeConditionalRetentionBudget {
        maximum_managed_clocks: 1,
        maximum_reserved_temporal_intents: 2,
        maximum_retained_definition_candidates: 1,
        maximum_retained_bytes: total,
        maximum_preparation_visits: 8,
    };
    let ledger = BridgeRetentionLedger::new(budget).unwrap();
    assert!(matches!(
        ledger.reserve(1, 3, total),
        Err(D::IntentsExhausted)
    ));
    assert_eq!(ledger.usage(), (0, 0, 0));
    let reservation = ledger.reserve(1, 2, total).unwrap();
    assert_eq!(ledger.usage(), (1, 2, total));
    assert!(matches!(ledger.reserve(1, 0, 0), Err(D::ClocksExhausted)));
    assert!(matches!(ledger.reserve(0, 0, 1), Err(D::BytesExhausted)));
    drop(reservation);
    let reservation = ledger.reserve(1, 2, total).unwrap();
    ledger.close();
    assert!(matches!(ledger.reserve(0, 0, 0), Err(D::Closed)));
    drop(reservation);
    assert_eq!(ledger.usage(), (0, 0, 0));
    let short = BridgeRetentionLedger::new(BridgeConditionalRetentionBudget {
        maximum_retained_bytes: total - 1,
        ..budget
    })
    .unwrap();
    assert!(matches!(short.reserve(1, 2, total), Err(D::BytesExhausted)));
    assert!(matches!(
        short.reserve(0, 0, u64::MAX),
        Err(D::BytesExhausted)
    ));
    assert_eq!(short.usage(), (0, 0, 0));
}

#[test]
fn aggregate_bytes_reject_overflow_without_disturbing_live_custody() {
    let ledger = BridgeRetentionLedger::new(BridgeConditionalRetentionBudget {
        maximum_retained_bytes: u64::MAX,
        ..BridgeConditionalRetentionBudget::development()
    })
    .unwrap();
    let held = ledger.reserve(0, 0, u64::MAX - 1).unwrap();
    assert!(matches!(ledger.reserve(0, 0, 2), Err(D::BytesExhausted)));
    assert_eq!(ledger.usage(), (0, 0, u64::MAX - 1));
    let final_byte = ledger.reserve(0, 0, 1).unwrap();
    assert_eq!(ledger.usage(), (0, 0, u64::MAX));
    drop(held);
    assert_eq!(ledger.usage(), (0, 0, 1));
    drop(final_byte);
    assert_eq!(ledger.usage(), (0, 0, 0));
}

#[test]
fn definition_candidate_slot_and_byte_limits_are_exact_and_recoverable() {
    let total = 257;
    let budget = BridgeConditionalRetentionBudget {
        maximum_retained_definition_candidates: 1,
        maximum_retained_bytes: total,
        ..BridgeConditionalRetentionBudget::development()
    };
    let ledger = BridgeRetentionLedger::new(budget).unwrap();
    let held = ledger.reserve_definition_candidate(total).unwrap();
    assert_eq!(ledger.observation().retained_definition_candidates(), 1);
    assert_eq!(ledger.observation().retained_bytes(), total);
    assert!(matches!(
        ledger.reserve_definition_candidate(0),
        Err(D::DefinitionCandidatesExhausted)
    ));
    drop(held);
    assert_eq!(ledger.observation().retained_definition_candidates(), 0);
    assert_eq!(ledger.observation().retained_bytes(), 0);

    let short = BridgeRetentionLedger::new(BridgeConditionalRetentionBudget {
        maximum_retained_bytes: total - 1,
        ..budget
    })
    .unwrap();
    assert!(matches!(
        short.reserve_definition_candidate(total),
        Err(D::BytesExhausted)
    ));
    assert_eq!(short.observation().retained_definition_candidates(), 0);
    assert_eq!(short.observation().retained_bytes(), 0);
}
