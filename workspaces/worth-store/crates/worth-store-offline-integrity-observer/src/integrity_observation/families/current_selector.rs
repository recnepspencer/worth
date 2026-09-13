use super::super::{OfflineIntegrityObservationCounters, OfflineIntegrityOutcome};
use super::{selector::read_selector, OfflineSelectorFacts, SelectorRole};
use worth_store_physical_format::integrity_declarations::families::root::CURRENT_SELECTOR_INTEGRITY_DECLARATION;

pub(crate) fn read_current_selector(
    bytes: &[u8],
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<OfflineSelectorFacts, OfflineIntegrityOutcome> {
    read_selector(
        bytes,
        SelectorRole::Current,
        CURRENT_SELECTOR_INTEGRITY_DECLARATION,
        counters,
    )
}
