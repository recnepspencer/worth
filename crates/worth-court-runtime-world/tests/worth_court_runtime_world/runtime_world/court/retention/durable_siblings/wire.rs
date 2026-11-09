//! Independent field locations in actual v5 frames for adversarial DATA ingress.
use super::super::owner_evidence::journey::{frame, rehash};
use super::*;
use std::ops::Range;
use worth_foundational::facade::WorldDurableJournal as Journal;
pub(super) type Rows = Vec<(Journal, u64, Vec<u8>)>;
pub(super) fn tag(bytes: &[u8]) -> u8 {
    bytes[52]
}
fn number(bytes: &[u8], at: usize) -> usize {
    usize::try_from(u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())).unwrap()
}
fn branch_end(bytes: &[u8], at: usize) -> usize {
    let length = u16::from_le_bytes(bytes[at + 24..at + 26].try_into().unwrap()) as usize;
    at + 26 + length
}
pub(super) fn association_end(bytes: &[u8]) -> usize {
    branch_end(bytes, 53) + 16
}
fn head_end(bytes: &[u8], at: usize) -> usize {
    branch_end(bytes, at) + 24
}
fn blob(bytes: &[u8], at: usize) -> Range<usize> {
    at + 8..at + 8 + number(bytes, at)
}
pub(super) struct Fields {
    pub custody: usize,
    pub source_generation: usize,
    pub intent_end: usize,
}
pub(super) fn fields(bytes: &[u8]) -> Fields {
    assert_eq!(&bytes[..8], b"WWRLDART");
    assert_eq!(&bytes[8..12], &5u32.to_le_bytes());
    assert!(matches!(tag(bytes), 7 | 8));
    let custody = association_end(bytes);
    let source_generation = branch_end(bytes, custody + 8) + 8;
    let mut at = head_end(bytes, head_end(bytes, custody + 8)) + 32;
    match bytes[at] {
        0 => at += 1,
        1 => at = blob(bytes, at + 1).end + 16,
        other => panic!("unknown Native creation tag {other}"),
    }
    match bytes[at] {
        0 => at += 1,
        1 => at = blob(bytes, at + 1).end,
        other => panic!("unknown Signal creation tag {other}"),
    }
    Fields {
        custody,
        source_generation,
        intent_end: at,
    }
}
pub(super) fn poison_reuse_owners(bytes: &mut Vec<u8>) {
    let fields = fields(bytes);
    let native = blob(bytes, fields.intent_end);
    let signal = blob(bytes, native.end);
    let bridge = blob(bytes, signal.end);
    let owners = bridge.end;
    assert_eq!(bytes[owners], 0, "actual exact reuse has no acting owner");
    let mut payload = bytes[52..owners].to_vec();
    payload.push(2);
    payload.extend_from_slice(&(signal.len() as u64).to_le_bytes());
    payload.extend_from_slice(&bytes[signal.clone()]);
    let maximum =
        worth_signal::facade::durability::SignalRecordedSourcePoint::MAXIMUM_ENCODED_BYTES;
    payload.resize(payload.len() + maximum - signal.len(), 0);
    payload.extend_from_slice(&bytes[owners + 1..]);
    *bytes = frame(b"WWRLDART", 5, &payload);
}
pub(super) fn release_from_intent(bytes: &[u8]) -> Vec<u8> {
    assert_eq!(tag(bytes), 7);
    let mut payload = vec![9];
    payload.extend_from_slice(&bytes[53..fields(bytes).custody + 8]);
    frame(b"WWRLDART", 5, &payload)
}
pub(super) fn refusal(
    fixture: &Fixture,
    request: &ExecutionRequest<'_, '_>,
    change: impl FnOnce(&mut Rows),
) -> WorldReadmissionDenial {
    PreparedWorldHomeData::from_home_source(
        fixture.recorder.binding().clone(),
        fixture.recorder.untrusted_snapshot(change),
        request,
    )
    .unwrap_err()
}
pub(super) fn repair(bytes: &mut [u8]) {
    rehash(bytes);
}
