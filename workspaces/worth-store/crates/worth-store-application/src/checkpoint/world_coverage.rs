//! Unclaimed durable World rows cannot be covered by a Native-only manifest.
use super::ApplicationHomeCheckpointDenial as Denial;
use worth_foundational::facade::WorldDurableJournal;
use worth_store::physical_runtime::{OrderedJournalName, ServingPhysicalRuntime};

pub(super) fn refuse_uncovered_world(serving: &ServingPhysicalRuntime) -> Result<(), Denial> {
    for journal in [
        WorldDurableJournal::Branches,
        WorldDurableJournal::Custody,
        WorldDurableJournal::Occurrences,
    ] {
        let journal = serving
            .ordered_journal(OrderedJournalName::new(journal.name()).expect("closed World journal"))
            .map_err(Denial::WorldJournalUnavailable)?;
        if journal.head().map_err(Denial::WorldJournalRead)?.is_some() {
            return Err(Denial::WorldSectionRequired);
        }
    }
    // This permits only the existing Native checkpoint lane. Empty journals
    // prove no Native/World reconciliation and grant no application service.
    Ok(())
}
