use std::collections::BTreeSet;

use super::{observe_artifact_at_path, DigestBuilder};

pub(super) struct SelectorEvidence {
    pub(super) count: u64,
    pub(super) linked_count: u64,
    pub(super) unpaired_count: u64,
    pub(super) digest: [u8; 32],
    pub(super) store_identity: Option<[u8; 16]>,
    pub(super) current_generation: Option<u64>,
}

pub(super) fn derive(files: &[(String, Vec<u8>)]) -> SelectorEvidence {
    let selectors = files
        .iter()
        .filter_map(|(path, bytes)| observe_artifact_at_path(path, bytes).selector)
        .collect::<Vec<_>>();
    let selector_ids = selectors
        .iter()
        .map(|selector| selector.identity)
        .collect::<BTreeSet<_>>();
    let store_identity = selectors.first().and_then(|first| {
        selectors
            .iter()
            .all(|selector| selector.store == first.store)
            .then_some(first.store)
    });
    let current_generation = selectors
        .iter()
        .find(|selector| selector.role == 1)
        .map(|selector| selector.generation);
    let linked_count = selectors
        .iter()
        .filter(|selector| selector.linked.is_some())
        .count() as u64;
    let unpaired_count = selectors
        .iter()
        .filter(|selector| {
            selector
                .linked
                .is_some_and(|identity| !selector_ids.contains(&identity))
        })
        .count() as u64;
    let mut digest = DigestBuilder::new(b"worth.store.recovery-observer.durable-selector.v1");
    for selector in &selectors {
        let linked = selector.linked.unwrap_or(0);
        let unpaired = selector
            .linked
            .is_some_and(|identity| !selector_ids.contains(&identity));
        let mut record = Vec::with_capacity(17);
        record.extend_from_slice(&selector.identity.to_le_bytes());
        record.extend_from_slice(&linked.to_le_bytes());
        record.push(u8::from(unpaired));
        digest.record(&record);
    }
    SelectorEvidence {
        count: selectors.len() as u64,
        linked_count,
        unpaired_count,
        digest: digest.finish().digest(),
        store_identity,
        current_generation,
    }
}
