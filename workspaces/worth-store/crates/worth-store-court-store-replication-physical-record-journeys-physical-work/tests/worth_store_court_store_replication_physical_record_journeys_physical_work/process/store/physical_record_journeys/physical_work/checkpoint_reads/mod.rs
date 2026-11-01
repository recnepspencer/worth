//! Actual filesystem cuts, not World/Native semantic admission.
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_store::physical_runtime::*;
use worth_store_testkit_store::fixture;
mod admission;
mod cut;
mod damage;
mod members;
mod retirement;
const JOURNALS: [&str; 3] = ["owner.first", "owner.second", "owner.third"];
fn anchor() -> AnchoredRecordName {
    AnchoredRecordName::new("home.checkpoint").unwrap()
}
fn registry() -> OrderedJournalRegistry {
    OrderedJournalRegistry::declare(3, &JOURNALS, &["home.checkpoint"]).unwrap()
}
fn initialize(root: &std::path::Path) -> ServingPhysicalRuntime {
    let (format, placement, access) = fixture::configuration();
    let media = fixture::media(root);
    let durability = fixture::durability(&media);
    fixture::success(
        media.initialize_record_store(
            PhysicalRecordInitialization::new(format, placement, access, durability)
                .with_journal_registry(registry()),
        ),
    )
}
fn reopen(root: &std::path::Path) -> ServingPhysicalRuntime {
    let (format, _, access) = fixture::configuration();
    let media = fixture::media(root);
    let durability = fixture::durability(&media);
    fixture::success(media.open_record_store(
        PhysicalRecordOpen::new(format, access, durability).with_journal_registry(registry()),
    ))
}
fn deadline() -> PhysicalMutationDeadline {
    PhysicalMutationDeadline::after_milliseconds(30_000).unwrap()
}
fn publish_anchor(
    serving: &ServingPhysicalRuntime,
    generation: u64,
    payload: Vec<u8>,
) -> AnchoredRecordPublished {
    serving
        .anchored_records()
        .unwrap()
        .publish(AnchoredRecordPublication::new(
            anchor(),
            generation,
            payload,
            fixture::placement(),
            deadline(),
        ))
        .unwrap()
}
fn publish_journals(serving: &ServingPhysicalRuntime, position: u64, bytes: &[u8]) {
    for name in JOURNALS {
        serving
            .ordered_journal(OrderedJournalName::new(name).unwrap())
            .unwrap()
            .publish(OrderedJournalPublication::new(
                position,
                bytes.to_vec(),
                fixture::placement(),
                deadline(),
            ))
            .unwrap();
    }
}
