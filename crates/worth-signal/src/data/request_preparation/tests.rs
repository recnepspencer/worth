use super::*;

#[test]
fn omitted_limit_preserves_ticket_growth_and_frame_refund() {
    let mut budget = SignalPreparationBudget::new(None);
    budget.claim_retained(1).unwrap();
    let mark = budget.checkpoint();
    budget.claim(65 << 20).unwrap();
    assert_eq!(budget.remaining(), None);
    assert_eq!(budget.memory.bytes(), 1 + (65 << 20));
    budget.release(mark);
    assert_eq!(budget.memory.bytes(), 1);
    budget.claim(65 << 20).unwrap();
    assert_eq!(budget.memory.bytes(), 1 + (65 << 20));
}

#[test]
fn explicit_zero_refuses_growth_without_changing_the_ticket() {
    let mut budget = SignalPreparationBudget::new(Some(0));
    assert!(matches!(
        budget.claim(1),
        Err(SignalError::PreparationMemoryExhausted {
            required: Some(1),
            reserved: Some(0),
        })
    ));
    assert_eq!(budget.memory.bytes(), 0);
    assert_eq!(budget.remaining(), Some(0));
}

#[test]
fn omitted_limit_still_refuses_counter_overflow_without_changing_the_ticket() {
    let mut budget = SignalPreparationBudget::new(None);
    budget.claim(u64::MAX).unwrap();
    assert!(matches!(
        budget.claim(1),
        Err(SignalError::PreparationMemoryExhausted {
            required: None,
            reserved: None,
        })
    ));
    assert_eq!(budget.memory.bytes(), u64::MAX);
}

#[test]
fn nested_scratch_release_preserves_diagnostic_custody() {
    let mut budget = SignalPreparationBudget::new(Some(1024));
    budget.claim_retained(16).unwrap();
    let epoch = budget.checkpoint();
    budget.reserve_diagnostics(512).unwrap();
    let scratch = budget.checkpoint();
    budget.claim(64).unwrap();
    budget.release(scratch);
    assert_eq!(budget.memory.bytes(), 528);
    budget.claim(400).unwrap();
    assert_eq!(budget.memory.bytes(), 928);
    budget.claim_diagnostics(512).unwrap();
    budget.release(epoch);
    assert_eq!(budget.memory.bytes(), 16);
}

#[test]
fn diagnostics_custody_cannot_be_consumed_by_unrelated_preparation() {
    let mut budget = SignalPreparationBudget::new(Some(1024));
    let epoch = budget.checkpoint();
    budget.reserve_diagnostics(512).unwrap();
    budget.claim(256).unwrap();
    assert_eq!(budget.memory.bytes(), 768);
    assert_eq!(budget.remaining(), Some(256));
    budget.claim_diagnostics(512).unwrap();
    assert_eq!(budget.memory.bytes(), 768);
    assert_eq!(budget.remaining(), Some(256));
    budget.release(epoch);
    assert_eq!(budget.memory.bytes(), 0);
}
