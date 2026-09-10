//! Unit evidence for undo admission (Gate 8.4 / R8.36 / R8.40 / R8.41).

use worth_foundational::facade::CanonicalDigestId;
use worth_query_installation::facade::PublishedAftermathPosture;

use super::aftermath_schema_fixture as fixture;
use super::undo_admission::derive_request_from_axes;
use super::WorthQueryUndoDenialKind;

// R8.41 — "Foundational material alone cannot authorize undo" — is a rung-1
// claim and is proved by `admit_undo`'s signature, which names only a
// move-only `WorthQueryRecoveryHandle` and a privately minted
// `WorthQueryRecoveryEffectAuthority`. Neither is Foundational material and
// neither is caller-constructible, so there is no lane to test at runtime.
// The compiler proof lives at
// `tests/ui/application_aftermath/foundational_material_cannot_admit_undo.rs`;
// the helper that used to stand in for it here only asserted that a function
// returning `FoundationalNotAuthority` returns `FoundationalNotAuthority`.

#[test]
fn undo_fails_closed_without_canonically_bound_original_input() {
    let denial = super::governed_input::require_original_governed_input(None)
        .expect_err("missing original input must deny");
    assert_eq!(
        denial.kind(),
        WorthQueryUndoDenialKind::OriginalGovernedInputRequired
    );
}

#[test]
fn irreversible_aftermath_denies_correction_not_admitted() {
    let aftermath = fixture::release_estate();
    assert_eq!(
        aftermath.published_posture(),
        PublishedAftermathPosture::Irreversible
    );
    let denied = derive_request_from_axes(&aftermath).expect_err("irreversible");
    assert_eq!(denied.kind(), WorthQueryUndoDenialKind::ReleasedEstate);
}

#[test]
fn compensatable_axis_derives_compensation_request() {
    let aftermath = fixture::transfer();
    assert_eq!(
        aftermath.published_posture(),
        PublishedAftermathPosture::Compensatable
    );
    let request = derive_request_from_axes(&aftermath).expect("derive");
    assert_eq!(request, super::WorthQueryUndoDerivedRequest::Compensation);
}

#[test]
fn reversible_axis_derives_recorded_inverse_request() {
    let aftermath = fixture::freeze_account();
    assert_eq!(
        aftermath.published_posture(),
        PublishedAftermathPosture::Reversible
    );
    let request = derive_request_from_axes(&aftermath).expect("derive");
    assert_eq!(
        request,
        super::WorthQueryUndoDerivedRequest::RecordedInverse
    );
}

#[test]
fn reconcilable_axis_derives_reconciliation_request() {
    let aftermath = fixture::notify_death();
    assert_eq!(
        aftermath.published_posture(),
        PublishedAftermathPosture::Reconcilable
    );
    let request = derive_request_from_axes(&aftermath).expect("derive");
    assert_eq!(request, super::WorthQueryUndoDerivedRequest::Reconciliation);
}

#[test]
fn irreversible_cause_kinds_are_distinguishable() {
    assert_eq!(
        derive_request_from_axes(&fixture::legal_hold())
            .unwrap_err()
            .kind(),
        WorthQueryUndoDenialKind::IrreversibleLegal
    );
    assert_eq!(
        derive_request_from_axes(&fixture::audit_retention())
            .unwrap_err()
            .kind(),
        WorthQueryUndoDenialKind::IrreversibleAudit
    );
    assert_eq!(
        derive_request_from_axes(&fixture::approve_emergency_access())
            .unwrap_err()
            .kind(),
        WorthQueryUndoDenialKind::IrreversibleApproval
    );
    assert_eq!(
        derive_request_from_axes(&fixture::release_estate())
            .unwrap_err()
            .kind(),
        WorthQueryUndoDenialKind::ReleasedEstate
    );
    assert_eq!(
        derive_request_from_axes(&fixture::wire_transfer_final())
            .unwrap_err()
            .kind(),
        WorthQueryUndoDenialKind::EscapedEffect
    );
}

#[test]
fn undo_admission_counters_stay_at_one_one_zero_across_axis_twins() {
    for (aftermath, expected) in [
        (
            fixture::transfer_small(),
            super::WorthQueryUndoDerivedRequest::Compensation,
        ),
        (
            fixture::transfer_large(),
            super::WorthQueryUndoDerivedRequest::Compensation,
        ),
    ] {
        assert_eq!(derive_request_from_axes(&aftermath).unwrap(), expected);
        assert_eq!(aftermath.canonical().basis_preparation_count(), 1);
        assert_eq!(aftermath.canonical().digest_derivation_count(), 1);
        assert_eq!(aftermath.canonical().digest_text_materializations(), 0);
    }
}

#[test]
fn terminal_and_expired_recovery_map_to_already_consumed_and_stale() {
    use super::recovery_handle::WorthQueryRecoveryHandleDenialKind as K;
    use super::undo_admission::map_recovery_denial;
    use super::undo_progression::map_ordinary_commit_conflict;

    assert_eq!(
        map_recovery_denial(K::AlreadyTerminal).kind(),
        WorthQueryUndoDenialKind::AlreadyConsumed
    );
    assert_eq!(
        map_recovery_denial(K::Expired).kind(),
        WorthQueryUndoDenialKind::Stale
    );
    assert_eq!(
        map_ordinary_commit_conflict().kind(),
        WorthQueryUndoDenialKind::Conflicted
    );
}

#[test]
fn undo_intent_identity_invariant_across_posting_and_lineage_fanout() {
    let aftermath_digest = CanonicalDigestId::new([0x44; 32]);
    let installed = [0x55; 32];
    let committed = crate::domain_computation::primary_graph::committed_recoverable_application();
    let mut digests = Vec::new();
    for (postings, lineage) in [(10usize, 1usize), (1000, 100)] {
        let _discarded_fanout = (postings, lineage);
        let intent = super::WorthQueryUndoIntentIdentity::derive_parts(
            committed.committed_product_publication(),
            installed,
            aftermath_digest,
            7,
        )
        .expect("intent");
        assert_eq!(intent.work().basis_preparations(), 1);
        assert_eq!(intent.work().digest_derivations(), 1);
        assert_eq!(intent.work().digest_text_materializations(), 0);
        digests.push(*intent.digest());
    }
    assert_eq!(
        digests[0], digests[1],
        "intent identity must be identical across posting/lineage fan-out twins"
    );
}
