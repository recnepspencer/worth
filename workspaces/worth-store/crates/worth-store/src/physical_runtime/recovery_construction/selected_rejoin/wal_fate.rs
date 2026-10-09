//! Store-side fate comparison over C.9-admitted frames reread from the same
//! selected media session. The tag-7 tip covers a pruned historical member.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, ReleasedDropTipProvenanceV1,
};

use crate::physical_runtime::{
    IntegrityAdmittedRecoveryWalFrame, StoreRecoveryBindingFreshnessSample,
    StoreRecoveryOperationFate,
};

const DROP_MATERIAL_DOMAIN: &[u8] = b"worth.store.blob.reclaim.mutation.v1";
const IDEMPOTENCY_DOMAIN: &[u8] = b"store.physical.mutation.idempotency-key.v1";

#[path = "wal_fate/pending.rs"]
mod pending;

pub(super) use pending::{
    matched_pending_projection, matches_pending_projected_set, matches_pending_release,
};

pub(super) fn matches_selected_fate(
    store: StableStoreIdentity,
    attempt: [u8; 16],
    tip: ReleasedDropTipProvenanceV1,
    sample: &StoreRecoveryBindingFreshnessSample,
    selected_wal: &[IntegrityAdmittedRecoveryWalFrame],
    checkpoint_cutoff: u64,
) -> bool {
    let request = tip.request();
    let witness = tip.fate();
    if sample.store_identity() != store || witness.lsn_end_exclusive() > checkpoint_cutoff {
        return false;
    }
    let mut members = sample
        .wal_members()
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
    let mut overlapping = selected_wal.iter().filter(|frame| {
        frame.lsn_start() < witness.lsn_end_exclusive() && witness.lsn_start() < frame.lsn_end()
    });
    let retained_frame = if let Some(frame) = overlapping.next() {
        if overlapping.next().is_some()
            || frame.lsn_start() != witness.lsn_start()
            || frame.lsn_end() != witness.lsn_end_exclusive()
            || frame.identity_digest() != witness.identity_digest()
            || frame.payload_digest() != witness.payload_digest()
        {
            return false;
        }
        true
    } else {
        false
    };
    let mut operations = sample
        .operations()
        .iter()
        .filter(|operation| operation.idempotency_identity() == request.idempotency());
    let expected_key = expected_drop_key(
        store.bytes(),
        attempt,
        sample.policy_identity(),
        request.lease_issuance_generation(),
        request.lease_expiry_generation(),
    );
    let Some(operation) = operations.next() else {
        // A normal cutover may prune every old C.9 fact. The exact selected
        // tag-7 certificate, rooted controls, and checkpoint are attested by
        // the caller; no surviving member/frame may contradict that history.
        return selected_history_fate_matches(None, retained_member, retained_frame)
            && expected_key == request.idempotency();
    };
    // An old V3 may remain raw C.9 Indeterminate after its result was
    // selected and certified by tag-7. Do not rewrite that fate. The
    // retained exact WAL frame must still exist in this independently sampled
    // inventory. C.9's tail member sample can omit a checkpoint-covered
    // member; when present, its identity/range was joined above. The selected
    // tag-7 certificate, controls, and root are checked by the caller.
    // An absent operation is handled separately as selected tag-7 custody.
    let admitted_fate =
        selected_history_fate_matches(Some(operation.fate()), retained_member, retained_frame);
    operations.next().is_none()
        && admitted_fate
        && operation.request_fingerprint().bytes() == request.fingerprint()
        && operation.lease_issuance_generation() == request.lease_issuance_generation()
        && operation.lease_expiry_generation() == request.lease_expiry_generation()
        && expected_key == request.idempotency()
}

fn selected_history_fate_matches(
    fate: Option<StoreRecoveryOperationFate>,
    retained_member: bool,
    retained_frame: bool,
) -> bool {
    match fate {
        None => !retained_member && !retained_frame,
        Some(StoreRecoveryOperationFate::AcknowledgedDurable)
        | Some(StoreRecoveryOperationFate::DurableUnacknowledged) => true,
        Some(StoreRecoveryOperationFate::Indeterminate) => retained_frame,
        _ => false,
    }
}

#[cfg(test)]
mod selected_history_tests {
    use super::{selected_history_fate_matches, StoreRecoveryOperationFate as Fate};

    #[test]
    fn fully_pruned_tag7_history_is_distinct_from_retained_conflict() {
        assert!(selected_history_fate_matches(None, false, false));
        assert!(!selected_history_fate_matches(None, true, false));
        assert!(!selected_history_fate_matches(None, false, true));
        assert!(!selected_history_fate_matches(None, true, true));
        assert!(!selected_history_fate_matches(
            Some(Fate::Indeterminate),
            false,
            false,
        ));
        assert!(selected_history_fate_matches(
            Some(Fate::Indeterminate),
            false,
            true,
        ));
        assert!(!selected_history_fate_matches(
            Some(Fate::ProvenNoEffect),
            false,
            false,
        ));
    }
}

pub(super) fn expected_drop_key(
    store: [u8; 16],
    attempt: [u8; 16],
    policy: [u8; 32],
    issuance: u64,
    expiry: u64,
) -> [u8; 32] {
    let mut material = Sha256::new();
    material.update(DROP_MATERIAL_DOMAIN);
    material.update(store);
    material.update(attempt);
    material.update([2]);
    let mut key = Sha256::new();
    key.update((IDEMPOTENCY_DOMAIN.len() as u64).to_le_bytes());
    key.update(IDEMPOTENCY_DOMAIN);
    key.update(store);
    key.update(policy);
    key.update(issuance.to_le_bytes());
    key.update(expiry.to_le_bytes());
    key.update(material.finalize());
    key.finalize().into()
}
