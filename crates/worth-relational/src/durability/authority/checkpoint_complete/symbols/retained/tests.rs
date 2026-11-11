//! Sparse internal separators and original-request scratch denial/refund.
use super::*;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
fn sparse() -> StringInterner {
    let mut source = StringInterner::default();
    for index in 0..160 {
        source.reserve_identity().unwrap();
        source.intern(&format!("entry-{index}")).unwrap();
    }
    source.reserve_frontier(u32::MAX - 1).unwrap();
    source.intern("last-valid-identity").unwrap();
    source
}
fn controller() -> u64 {
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let mut charge = 0;
    ExecutionRequest::serial(&host)
        .run_native_with_checkpoint(|_| charge = budget.charged_memory_bytes())
        .unwrap();
    assert_eq!(budget.charged_memory_bytes(), 0);
    charge
}
#[test]
fn funded_sparse_dictionary_traversal_matches_original_keys_and_refunds() {
    let source = sparse();
    let expected = source
        .retained_entries()
        .map(|(key, text)| (*key, text.as_str()))
        .collect::<Vec<_>>();
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let mut entries = RetainedSymbols::new(&source, &ExecutionRequest::serial(&host)).unwrap();
    assert!(budget.charged_memory_bytes() > 0);
    for expected in expected {
        let (key, text) = entries.next().unwrap();
        assert_eq!((*key, text.as_str()), expected);
    }
    assert!(entries.next().is_none());
    assert!(budget.charged_memory_bytes() > 0);
    drop(entries);
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(
        source.resolve(Symbol(u32::MAX - 1)),
        Some("last-valid-identity")
    );
}
#[test]
fn dictionary_scratch_bound_refuses_before_iterator_construction() {
    let source = sparse();
    // 161 entries bound depth by eight: two eight-frame paths plus four old
    // frames during realloc. Frame layout is independently a thin ref+usize.
    let bill = std::alloc::Layout::array::<(&(), usize)>(20)
        .unwrap()
        .size() as u64;
    let budget = SerialMemoryBudget::new(Some(controller() + bill - 1));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&host);
    assert!(
        matches!(RetainedSymbols::new(&source, &request), Err(Denial::Request(crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {requested, admitted, ancestor:0})) if requested == bill && admitted == bill - 1)
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
    cancellation.cancel();
    assert!(matches!(
        RetainedSymbols::new(&source, &request),
        Err(Denial::Request(
            crate::execution::RelationalExecutionDenialCause::Cancelled
        ))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(source.retained_entry_count(), 161);
}
