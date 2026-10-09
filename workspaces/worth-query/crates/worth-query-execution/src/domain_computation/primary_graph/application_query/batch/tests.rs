use super::*;

fn limits(work: usize, bytes: usize) -> WorthQueryApplicationQueryBatchLimits {
    WorthQueryApplicationQueryBatchLimits::new(
        NonZeroUsize::new(8).unwrap(),
        NonZeroUsize::new(work).unwrap(),
        NonZeroUsize::new(bytes).unwrap(),
        NonZeroUsize::new(2048).unwrap(),
    )
}

#[test]
fn repeated_reads_spend_one_loan_and_refund_only_their_unused_share() {
    let batch = WorthQueryApplicationQueryBatchAdmission::new(limits(10, 32));
    let first = batch.reserve_read(8).unwrap();
    assert_eq!(first.maximum(), 8);
    first.settle(4).unwrap();
    let second = batch.reserve_read(8).unwrap();
    assert_eq!(
        second.maximum(),
        6,
        "a second read cannot renew the original ten units"
    );
    second.settle(6).unwrap();
    assert!(matches!(
        batch.reserve_read(1),
        Err(WorthQueryApplicationQueryBatchResourceDenial::WorkLimit { maximum: 10, .. })
    ));
    assert_eq!(batch.observe().read_work_units(), 10);
}

#[test]
fn failed_settlement_and_overflow_leave_existing_reservations_charged() {
    let batch = WorthQueryApplicationQueryBatchAdmission::new(limits(10, usize::MAX));
    let reserved = batch.reserve_read(6).unwrap();
    assert_eq!(
        reserved.settle(7),
        Err(WorthQueryApplicationQueryBatchResourceDenial::WorkAccountingMismatch)
    );
    assert_eq!(batch.observe().read_work_units(), 6);
    let old = batch.claim_memory(usize::MAX).unwrap();
    assert!(matches!(
        batch.claim_memory(1),
        Err(WorthQueryApplicationQueryBatchResourceDenial::CounterOverflow)
    ));
    assert_eq!(batch.observe().retained_bytes(), usize::MAX);
    drop(old);
    assert_eq!(batch.observe().retained_bytes(), 0);
}

#[test]
fn a_late_shared_buffer_refusal_keeps_prior_source_custody_until_drop() {
    use super::super::resource_lifecycle::WorthQueryApplicationResultBufferRegistry;
    let registry = WorthQueryApplicationResultBufferRegistry::default();
    let batch = WorthQueryApplicationQueryBatchAdmission::new(limits(10, 31));
    let mut first = registry.reserve_in_batch(32, &batch);
    first.claim(12).unwrap();
    let source = first.claim_retained_source(11).unwrap();
    first.release();
    assert_eq!(batch.observe().retained_bytes(), 11);
    let mut second = registry.reserve_in_batch(32, &batch);
    second.claim(20).unwrap();
    assert!(second.claim_retained_source(1).is_err());
    assert_eq!(batch.observe().retained_bytes(), 31);
    assert_eq!(registry.observer().observe().retained_bytes(), 31);
    drop(second);
    assert_eq!(batch.observe().retained_bytes(), 11);
    drop(source);
    assert_eq!(batch.observe().retained_bytes(), 0);
    assert_eq!(registry.observer().observe().retained_bytes(), 0);
    assert_eq!(registry.observer().observe().active_buffers(), 0);
}
