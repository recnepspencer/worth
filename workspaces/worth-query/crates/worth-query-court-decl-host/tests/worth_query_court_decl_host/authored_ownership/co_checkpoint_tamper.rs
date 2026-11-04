//! Corrupt one recorded co-anchor through the genuine checkpoint framing owner.
use super::native_checkpoint_tamper::{field, items};
use super::*;
pub(super) fn tamper(
    checkpoint: &WorthQueryApplicationCheckpoint,
) -> WorthQueryApplicationCheckpoint {
    super::native_checkpoint_tamper::mutate(checkpoint, |envelope| {
        let group = field(envelope, "authored_cascade_retirement");
        if envelope[group.clone()] == [0xc0] {
            return None;
        }
        let co = field(&envelope[group.clone()], "co_anchors");
        let anchors = &envelope[group.clone()][co.clone()];
        let first = items(anchors)[0].clone();
        let id = items(&anchors[first.clone()])[0].clone();
        let slot = field(&anchors[first.clone()][id.clone()], "local_slot");
        let offset = group.start + co.start + first.start + id.start + slot.start;
        assert!(envelope[offset] < 128 && envelope[offset] != 127);
        Some((offset..offset + 1, vec![127]))
    })
}
