//! Selected C.9 WAL-member/fate join for a checkpoint-carried released drop.

use worth_store_physical_format::{OriginalDropReservationRequestV1, ReleasedDropWalFateWitnessV1};
use worth_store_recovery_physics::{ReconciledOperationFates, RecoveryOperationFate};

use super::SelectedReleaseJoin;
use crate::orchestration::planning::completion::blob_reclaim::manifest_residue::wal_fate::expected_drop_key;

impl SelectedReleaseJoin<'_> {
    pub(super) fn durable_fate(
        &self,
        request: OriginalDropReservationRequestV1,
        attempt: [u8; 16],
        witness: ReleasedDropWalFateWitnessV1,
    ) -> bool {
        if witness.lsn_end_exclusive() > self.checkpoint_cutoff {
            return false;
        }
        let mut members = self
            .selected_members
            .iter()
            .filter(|member| member.operation_identity() == request.idempotency());
        let retained_member = if let Some(member) = members.next() {
            let range = member.lsn_range();
            if members.next().is_some()
                || range.start().get() != witness.lsn_start()
                || range.end_exclusive().get() != witness.lsn_end_exclusive()
            {
                return false;
            }
            true
        } else {
            false
        };
        if !retained_wal_matches(
            witness,
            self.selected_wal.iter().map(|frame| {
                (
                    frame.lsn_start(),
                    frame.lsn_end(),
                    frame.identity_digest(),
                    frame.payload_digest(),
                )
            }),
        ) {
            return false;
        }
        // An Indeterminate C.9 operation is not promoted here. The selected
        // tag-7 Batch/Accumulator and exact controls certify the published
        // result, but while its WAL frame is retained we also require the
        // frame's exact interval and both digests before accepting that fate.
        let retained_exact = self.selected_wal.iter().any(|frame| {
            frame.lsn_start() == witness.lsn_start()
                && frame.lsn_end() == witness.lsn_end_exclusive()
                && frame.identity_digest() == witness.identity_digest()
                && frame.payload_digest() == witness.payload_digest()
        });
        // If the C.9 member was reclaimed, these fields are selected tag-7
        // Store custody, not an independently replayed historical WAL frame.
        operation_fate_matches(
            self.fates,
            request,
            self.store,
            attempt,
            self.policy,
            retained_exact,
            retained_member,
        )
    }
}

fn retained_wal_matches(
    witness: ReleasedDropWalFateWitnessV1,
    frames: impl IntoIterator<Item = (u64, u64, [u8; 32], [u8; 32])>,
) -> bool {
    let mut overlaps = frames.into_iter().filter(|(start, end, _, _)| {
        *start < witness.lsn_end_exclusive() && witness.lsn_start() < *end
    });
    let Some((start, end, identity, payload)) = overlaps.next() else {
        return true; // selected checkpoint custody must still join separately
    };
    overlaps.next().is_none()
        && start == witness.lsn_start()
        && end == witness.lsn_end_exclusive()
        && identity == witness.identity_digest()
        && payload == witness.payload_digest()
}

fn operation_fate_matches(
    fates: &ReconciledOperationFates,
    request: OriginalDropReservationRequestV1,
    store: [u8; 16],
    attempt: [u8; 16],
    policy: [u8; 32],
    retained_exact: bool,
    retained_member: bool,
) -> bool {
    let mut matching = fates
        .operations()
        .iter()
        .filter(|operation| operation.identity().idempotency() == request.idempotency());
    let expected_key = expected_drop_key(
        store,
        attempt,
        policy,
        request.lease_issuance_generation(),
        request.lease_expiry_generation(),
    );
    let Some(operation) = matching.next() else {
        // A selected tag-7 checkpoint is Store-attested custody even after
        // normal cutover prunes the entire historical C.9 interval. No
        // surviving member/frame may conflict with that certified history.
        return !retained_exact && !retained_member && expected_key == request.idempotency();
    };
    matching.next().is_none()
        && operation.identity().store() == store
        && (matches!(
            operation.fate(),
            RecoveryOperationFate::AcknowledgedDurable
                | RecoveryOperationFate::DurableUnacknowledged
        ) || (retained_exact && operation.fate() == RecoveryOperationFate::Indeterminate))
        && operation.request_fingerprint() == request.fingerprint()
        && operation.lease_issuance_generation() == request.lease_issuance_generation()
        && operation.lease_expiry_generation() == request.lease_expiry_generation()
        && expected_key == request.idempotency()
}

#[cfg(test)]
mod tests {
    use worth_store_recovery_physics::{
        reconcile_operation_fates, RecoveryBindingFreshness, RecoveryOperationEvidenceInput,
        RecoveryOperationIdentity,
    };

    use super::*;

    #[test]
    fn retained_c9_member_requires_exact_interval_and_both_digests() {
        let witness = ReleasedDropWalFateWitnessV1::new(10, 20, [3; 32], [4; 32]).unwrap();
        let good = (10, 20, [3; 32], [4; 32]);
        assert!(retained_wal_matches(witness, [good]));
        assert!(!retained_wal_matches(witness, [(10, 20, [9; 32], [4; 32])]));
        assert!(!retained_wal_matches(witness, [(10, 20, [3; 32], [9; 32])]));
        assert!(!retained_wal_matches(witness, [(11, 20, [3; 32], [4; 32])]));
        assert!(!retained_wal_matches(witness, [good, good]));
        assert!(!retained_wal_matches(witness, [(9, 11, [3; 32], [4; 32])]));
    }

    #[test]
    fn original_drop_requires_unique_durable_matching_operation() {
        let store = [1; 16];
        let attempt = [2; 16];
        let policy = [3; 32];
        let key = expected_drop_key(store, attempt, policy, 1, 4);
        let request = OriginalDropReservationRequestV1::new(key, [5; 32], 1, 4).unwrap();
        let fates = |state, duplicate, fingerprint| {
            let operation = |ordinal| {
                RecoveryOperationEvidenceInput::new(
                    RecoveryOperationIdentity::new(store, ordinal, 1, 1, key).unwrap(),
                    fingerprint,
                    1,
                    4,
                    RecoveryBindingFreshness::Retained,
                    state,
                )
            };
            let mut inputs = vec![operation(1)];
            if duplicate {
                inputs.push(operation(2));
            }
            reconcile_operation_fates(3, inputs, 2).unwrap()
        };
        assert!(operation_fate_matches(
            &fates(RecoveryOperationFate::AcknowledgedDurable, false, [5; 32]),
            request,
            store,
            attempt,
            policy,
            false,
            false,
        ));
        assert!(operation_fate_matches(
            &fates(RecoveryOperationFate::Indeterminate, false, [5; 32]),
            request,
            store,
            attempt,
            policy,
            true,
            false,
        ));
        assert!(!operation_fate_matches(
            &fates(RecoveryOperationFate::Indeterminate, false, [5; 32]),
            request,
            store,
            attempt,
            policy,
            false,
            false,
        ));
        assert!(!operation_fate_matches(
            &fates(RecoveryOperationFate::ProvenNoEffect, false, [5; 32]),
            request,
            store,
            attempt,
            policy,
            false,
            false,
        ));
        assert!(!operation_fate_matches(
            &fates(RecoveryOperationFate::AcknowledgedDurable, false, [6; 32]),
            request,
            store,
            attempt,
            policy,
            false,
            false,
        ));
        assert!(!operation_fate_matches(
            &fates(RecoveryOperationFate::AcknowledgedDurable, true, [5; 32]),
            request,
            store,
            attempt,
            policy,
            false,
            false,
        ));
        let pruned = reconcile_operation_fates(3, Vec::new(), 2).unwrap();
        assert!(operation_fate_matches(
            &pruned, request, store, attempt, policy, false, false,
        ));
        assert!(!operation_fate_matches(
            &pruned, request, store, attempt, policy, true, false,
        ));
        assert!(!operation_fate_matches(
            &pruned, request, store, attempt, policy, false, true,
        ));
    }
}
