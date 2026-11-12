//! Floors come only from the actual originally published canonical event DATA.
use worth_relational::facade::history::CanonicalCommitEnvelope;
pub(super) fn published_floor(envelope: &CanonicalCommitEnvelope) -> (u64, u64) {
    let node = envelope
        .lineage_events()
        .iter()
        .flat_map(|event| event.targets())
        .map(|id| id.0)
        .max()
        .unwrap()
        + 1;
    let event = envelope
        .lineage_events()
        .iter()
        .map(|event| event.event_id())
        .max()
        .unwrap()
        + 1;
    (node, event)
}
