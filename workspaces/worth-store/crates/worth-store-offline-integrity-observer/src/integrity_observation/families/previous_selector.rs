use super::super::{OfflineIntegrityObservationCounters, OfflineIntegrityOutcome};
use super::{selector::read_selector, OfflineSelectorFacts, SelectorRole};
use worth_store_physical_format::integrity_declarations::families::root::PREVIOUS_SELECTOR_INTEGRITY_DECLARATION;

pub(crate) fn read_previous_selector(
    bytes: &[u8],
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<OfflineSelectorFacts, OfflineIntegrityOutcome> {
    read_selector(
        bytes,
        SelectorRole::Previous,
        PREVIOUS_SELECTOR_INTEGRITY_DECLARATION,
        counters,
    )
}
