//! A tier-epoch WAL intent is retained maintenance, never a data group.

use std::collections::BTreeMap;
use worth_store_physical_format::{TierEpochActivationPhaseV1, TierEpochActivationV1};
use worth_store_wal::WalLsnRange;

use super::{PhysicalWalBindingReopenCutoff, PhysicalWalOpenFailure};

pub(super) fn observe(
    payload: &[u8],
    range: WalLsnRange,
    segment: (u64, u64),
    store: [u8; 16],
    cutoff: PhysicalWalBindingReopenCutoff,
    retained_spans: &mut Vec<(u64, u64)>,
    root_metadata: &mut BTreeMap<u64, (u64, Option<(u64, u64)>)>,
) -> Result<(), PhysicalWalOpenFailure> {
    let activation = TierEpochActivationV1::decode(payload)
        .map_err(|_| PhysicalWalOpenFailure::MemberPayloadRejected)?;
    if activation.store() != store {
        return Err(PhysicalWalOpenFailure::MemberPayloadRejected);
    }
    let root = activation.candidate_root_generation();
    let bytes = activation.retained_metadata_bytes();
    let entry = root_metadata.entry(root).or_insert((bytes, None));
    if entry.0 != bytes {
        return Err(PhysicalWalOpenFailure::MemberPayloadRejected);
    }
    if activation.phase() == TierEpochActivationPhaseV1::Intent && entry.1.is_none() {
        entry.1 = Some(segment);
    }
    match cutoff.lsn() {
        Some(cutoff_lsn) if range.end_exclusive() <= cutoff_lsn => {}
        Some(cutoff_lsn) if range.start() < cutoff_lsn => {
            return Err(PhysicalWalOpenFailure::MemberPayloadRejected);
        }
        _ => retained_spans.push((range.start().get(), range.end_exclusive().get())),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_wal::LogSequenceNumber;

    #[test]
    fn observe_records_intent_metadata_and_rejects_mismatched_completion() {
        let store = [7u8; 16];
        let intent = TierEpochActivationV1::intent(
            store, [1u8; 16], 1, [2u8; 32], [3u8; 32], 4, 2, [5u8; 32], 4_096, 9,
        )
        .unwrap();
        let range = WalLsnRange::new(LogSequenceNumber::new(1), LogSequenceNumber::new(2)).unwrap();
        let mut spans = Vec::new();
        let mut meta = BTreeMap::new();
        observe(
            &intent.encode(),
            range,
            (1, 1),
            store,
            PhysicalWalBindingReopenCutoff::GenerationZero,
            &mut spans,
            &mut meta,
        )
        .unwrap();
        assert_eq!(meta.get(&2), Some(&(4_096, Some((1, 1)))));
        let range2 =
            WalLsnRange::new(LogSequenceNumber::new(2), LogSequenceNumber::new(3)).unwrap();
        observe(
            &intent.completed().encode(),
            range2,
            (2, 1),
            store,
            PhysicalWalBindingReopenCutoff::GenerationZero,
            &mut spans,
            &mut meta,
        )
        .unwrap();
        assert_eq!(meta.get(&2), Some(&(4_096, Some((1, 1)))));
        let bad = TierEpochActivationV1::intent(
            store, [1u8; 16], 1, [2u8; 32], [3u8; 32], 4, 2, [5u8; 32], 8_192, 9,
        )
        .unwrap()
        .completed();
        assert!(observe(
            &bad.encode(),
            range2,
            (2, 1),
            store,
            PhysicalWalBindingReopenCutoff::GenerationZero,
            &mut spans,
            &mut meta,
        )
        .is_err());
    }
}
