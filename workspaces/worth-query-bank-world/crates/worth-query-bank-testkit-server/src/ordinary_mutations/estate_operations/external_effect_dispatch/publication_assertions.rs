//! Public-consumer assertions for closed external-effect publication.

pub use bank_server::{
    BankCommitCanonicalWorkEvidence, BankCommitCanonicalWorkPhases, BankCommitReceipt,
};
pub use worth_query_host::facade::publication::application_aftermath::{
    WorthQueryPublishedApplicationCommitKind, WorthQueryPublishedExternalEffectPostureKind,
};

pub fn assert_recovered_commit_axes(executed: &BankCommitReceipt, recovered: &BankCommitReceipt) {
    assert_eq!(
        executed.publication().inspect().kind(),
        WorthQueryPublishedApplicationCommitKind::Executed
    );
    assert_eq!(
        recovered.publication().inspect().kind(),
        WorthQueryPublishedApplicationCommitKind::Recovered
    );
    assert_eq!(
        executed.aftermath().posture(),
        recovered.aftermath().posture()
    );
    assert_eq!(
        executed.changed_record_count(),
        recovered.changed_record_count()
    );
    assert_eq!(
        executed.emitted_effect_count(),
        recovered.emitted_effect_count()
    );
    assert_eq!(
        executed.expected_version_count(),
        recovered.expected_version_count()
    );
    assert_eq!(
        executed.expected_fact_count(),
        recovered.expected_fact_count()
    );
    assert_eq!(
        executed.decision_fact_count(),
        recovered.decision_fact_count()
    );
    let executed_work = executed.canonical_work();
    let recovered_work = recovered.canonical_work();
    assert_recovered_non_dispatch_work(executed_work, recovered_work);
    assert!(executed_work.external_dispatch().basis_preparations() > 0);
    assert_eq!(
        recovered_work.external_dispatch(),
        BankCommitCanonicalWorkEvidence::default()
    );
    assert_eq!(
        executed.co_committed_dispatch_outbox(),
        recovered.co_committed_dispatch_outbox()
    );
    assert_eq!(executed.retained_preimage(), recovered.retained_preimage());
    assert_eq!(
        executed.performed_preimage_retention_work(),
        recovered.performed_preimage_retention_work()
    );
}

pub fn assert_recovered_non_dispatch_work(
    executed: BankCommitCanonicalWorkPhases,
    recovered: BankCommitCanonicalWorkPhases,
) {
    for (executed_phase, recovered_phase) in [
        (executed.installation(), recovered.installation()),
        (executed.admission(), recovered.admission()),
        (executed.execution(), recovered.execution()),
        (executed.provider_commit(), recovered.provider_commit()),
        (executed.projection(), recovered.projection()),
        (executed.live_delivery(), recovered.live_delivery()),
        (executed.retry_resolution(), recovered.retry_resolution()),
        (
            executed.recovery_inspection(),
            recovered.recovery_inspection(),
        ),
        (executed.publication(), recovered.publication()),
        (executed.undo_admission(), recovered.undo_admission()),
        (executed.redo_admission(), recovered.redo_admission()),
    ] {
        assert_eq!(executed_phase, recovered_phase);
    }
}

pub fn assert_dispatch_publication_and_work(receipt: &BankCommitReceipt, scenario: &str) {
    let posture = receipt
        .external_dispatch_posture()
        .expect("a dispatched effect has a published posture");
    let event_count = match posture.kind() {
        WorthQueryPublishedExternalEffectPostureKind::Completed
        | WorthQueryPublishedExternalEffectPostureKind::Acknowledged => 4,
        WorthQueryPublishedExternalEffectPostureKind::Unresolved => 3,
        WorthQueryPublishedExternalEffectPostureKind::NotDeclared
        | WorthQueryPublishedExternalEffectPostureKind::PendingDispatch => {
            panic!("{scenario}: a dispatched effect cannot publish as undispatched")
        }
    };
    let work = receipt.canonical_work().external_dispatch();
    assert_eq!(work.basis_preparations(), event_count);
    assert_eq!(work.digest_derivations(), event_count);
    assert_eq!(work.digest_text_materializations(), 0);
    assert!(
        work.canonical_encoded_bytes() > 0,
        "{scenario}: causal identities have canonical bytes"
    );
}
