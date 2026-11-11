//! D1647: a real refusal ends Native's symbol probe without another callback.
use worth_relational::facade::inspection::{
    MaintenanceWorkObservation, SymbolPreparationEvent as Event,
};
pub(super) fn assert_refusal_stops(observation: &MaintenanceWorkObservation) {
    assert_eq!(observation.omitted_symbol_preparation_events, 0);
    let events = &observation.symbol_preparation;
    let stopped = events
        .iter()
        .position(|event| *event == Event::AssemblerBreak)
        .expect("the actual Native assembler observed a real Break");
    assert!(
        events[..stopped].contains(&Event::ColdBucketRetry),
        "the refusal journey must cross a real cold-symbol retry"
    );
    assert!(events[..stopped].contains(&Event::AssemblerCallback));
    let after = &events[stopped + 1..];
    assert_eq!(
        after
            .iter()
            .filter(|event| **event == Event::AssemblerCallback)
            .count(),
        0,
        "there can be no cold callback after the first real refusal"
    );
    assert_eq!(after, [Event::CoupledCompleted(false), Event::ProbeReturnedOk],
        "the actual completion returned Ok(false), and the symbol probe returned Ok without a retry or another round");
    eprintln!("D1647 real Native termination trace: {events:?}");
}
