use super::super::PhysicalResidencyAllocationBoundaryFacts as Facts;
use super::*;
use crate::PhysicalResidencyDimension;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

fn event(sequence: u64) -> Event {
    Event::new(
        sequence,
        Facts::release(PhysicalResidencyDimension::PinLeases, None, 1),
    )
}

#[test]
fn prepared_page_preserves_chronological_history_and_original_grant_until_trace_reclaim() {
    let mut trace = EventTrace::default();
    for sequence in 1..=128 {
        trace.push(event(sequence));
    }
    let original_newest = std::ptr::from_ref(trace.newest.as_deref().unwrap());
    let budget = SerialMemoryBudget::new(Some(1 << 20));
    let observer = budget.clone();
    let serial = SerialRequest::from_memory(budget, CancellationSource::new().token(), None);
    let mut page = PreparedTracePage::new(&ExecutionRequest::serial(&serial)).unwrap();
    assert!(observer.charged_memory_bytes() > 0);
    trace.prepare_capacity(3, &mut page);
    assert_eq!(
        std::ptr::from_ref(trace.newest.as_ref().unwrap().previous.as_deref().unwrap()),
        original_newest
    );
    for sequence in 129..=131 {
        trace.push_prepared(event(sequence));
    }
    assert_eq!(trace.snapshot(), (1..=131).map(event).collect::<Vec<_>>());
    drop(page);
    drop(serial);
    assert!(
        observer.charged_memory_bytes() > 0,
        "the actual appended page owns the original request admission"
    );
    drop(trace);
    assert_eq!(observer.charged_memory_bytes(), 0);
}

#[test]
fn denied_page_admission_preserves_existing_events_without_growth() {
    let mut trace = EventTrace::default();
    for sequence in 1..=64 {
        trace.push(event(sequence));
    }
    let before = trace.snapshot();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(0)),
        CancellationSource::new().token(),
        None,
    );
    assert!(PreparedTracePage::new(&ExecutionRequest::serial(&serial)).is_err());
    assert_eq!(trace.snapshot(), before);
    assert_eq!(trace.remaining(), 0);
    assert_eq!(serial.memory().unwrap().charged_memory_bytes(), 0);
}
