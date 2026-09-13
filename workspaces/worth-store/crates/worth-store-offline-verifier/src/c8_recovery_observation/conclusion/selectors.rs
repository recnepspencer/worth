use std::collections::BTreeSet;

use super::super::observer_evidence::RecoveryObserverSelectorEvidence;
use super::super::observer_evidence_accumulation::{
    EvidenceDigestBuilder, RecoveryObserverSelectorObservation,
};

pub(super) fn summarize(
    selectors: Vec<RecoveryObserverSelectorObservation>,
) -> RecoveryObserverSelectorEvidence {
    let selector_count = selectors.len() as u64;
    let selector_ids: BTreeSet<u64> = selectors.iter().map(|selector| selector.identity).collect();
    let store_identity = selectors.first().and_then(|selector| {
        selectors
            .iter()
            .all(|candidate| candidate.store_identity == selector.store_identity)
            .then_some(selector.store_identity)
    });
    let current_root_generation = selectors
        .iter()
        .find(|selector| selector.role == 1)
        .map(|selector| selector.root_generation);
    let mut selector_digest =
        EvidenceDigestBuilder::new(b"worth.store.recovery-observer.durable-selector.v1");
    let mut linked_selector_count = 0_u64;
    let mut unpaired_link_count = 0_u64;
    for selector in selectors {
        let linked = selector.linked_identity;
        let unpaired = linked.is_some_and(|identity| !selector_ids.contains(&identity));
        linked_selector_count = linked_selector_count.saturating_add(u64::from(linked.is_some()));
        unpaired_link_count = unpaired_link_count.saturating_add(u64::from(unpaired));
        let mut record = Vec::with_capacity(17);
        record.extend_from_slice(&selector.identity.to_le_bytes());
        record.extend_from_slice(&linked.unwrap_or(0).to_le_bytes());
        record.push(u8::from(unpaired));
        selector_digest.record(&record);
    }
    RecoveryObserverSelectorEvidence::from_parts(
        selector_count,
        linked_selector_count,
        unpaired_link_count,
        store_identity,
        current_root_generation,
        selector_digest.finish().digest(),
    )
}
