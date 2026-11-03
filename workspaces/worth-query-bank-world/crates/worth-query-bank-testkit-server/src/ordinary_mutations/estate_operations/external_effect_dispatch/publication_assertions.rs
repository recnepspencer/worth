//! Public-consumer assertions for closed external-effect publication.

pub use bank_server::{
    BankCommitCanonicalWorkEvidence, BankCommitCanonicalWorkPhases, BankCommitReceipt,
};
pub use worth_query_host::facade::publication::application_aftermath::{
    WorthQueryPublishedApplicationCommitKind, WorthQueryPublishedExternalEffectPostureKind,
};

/// Compare immutable original meaning with the independently held performed source.
/// Live work/dispatch counters are observed only on that original owner.
pub fn assert_recovered_commit_axes<R>(
    executed: &bank_server::BankClosedSourceCompletion<R>,
    recovered: &bank_server::BankClosedSourceCompletion<R>,
) {
    let live = executed
        .completion()
        .live_receipt()
        .expect("the original source owns actual live evidence");
    let data = recovered
        .recorded_receipt()
        .expect("retry reads recorded original meaning");
    assert!(recovered.live_receipt().is_none());
    assert!(recovered.fresh_result().is_none());
    assert_eq!(live.commit_reference(), data.commit_reference());
    assert_eq!(live.outcome_identity(), Some(data.outcome_identity()));
    assert_eq!(live.changed_record_count(), data.changed_record_count());
    assert_eq!(live.emitted_effect_count(), data.emitted_effect_count());
    assert_eq!(
        live.idempotency_binding().key_identity(),
        data.idempotency_key_identity()
    );
    let before = live.basis_descriptor();
    let after = data.basis_descriptor();
    assert_eq!(before.descriptor_version(), after.descriptor_version());
    assert_eq!(before.branch_id(), after.branch_id());
    assert_eq!(before.posture(), after.posture());
    assert_eq!(
        before.reference().generation(),
        after.reference().generation()
    );
    match (
        before.reference().target().as_basis(),
        after.reference().target().as_basis(),
    ) {
        (Some(left), Some(right)) => {
            assert_eq!(left.selected_commit_id(), right.selected_commit_id());
            assert_eq!(left.version_id(), right.version_id());
            assert_eq!(left.parent_commit_ids(), right.parent_commit_ids());
            assert_eq!(left.roots(), right.roots());
        }
        (None, None) => {}
        (Some(_), None) | (None, Some(_)) => panic!("original target kind changed"),
    }
    assert_eq!(before.truth_version(), after.truth_version());
    assert_eq!(before.schema_commitment(), after.schema_commitment());
    assert_eq!(
        before.visibility_commitment(),
        after.visibility_commitment()
    );
    let publication = live.committed_product_publication();
    let head = data
        .new_product_head()
        .memory()
        .expect("these fixtures own actual memory Homes");
    assert_eq!(
        publication.product_branch().owner_identity().get(),
        head.owner()
    );
    assert_eq!(publication.product_branch().name().as_str(), head.branch());
    assert_eq!(
        publication.product_incarnation().ordinal(),
        head.incarnation()
    );
    assert_eq!(publication.product_generation().get(), head.generation());
    assert_eq!(
        publication.composite_commit().ordinal(),
        head.selected_commit()
    );
    assert_eq!(
        publication.publication_attempt().ordinal(),
        data.attempt_identity().ordinal()
    );
    assert_eq!(
        live.recorded_output_settlement(),
        data.exact_output_settlement()
    );
    match (live.dispatch_outbox(), data.dispatch_outbox()) {
        (None, None) => {}
        (Some(original), Some(recorded)) => {
            assert_eq!(original.correlation(), recorded.correlation());
            assert_eq!(
                original.correlation_family().as_str(),
                recorded.correlation_family()
            );
            assert_eq!(original.effect(), recorded.effect());
            assert_eq!(original.protocol_identity(), recorded.protocol_identity());
            assert_eq!(original.protocol_version(), recorded.protocol_version());
            assert_eq!(
                original.maximum_payload_bytes(),
                recorded.maximum_payload_bytes()
            );
            assert_eq!(original.payload(), recorded.payload());
            assert_eq!(original.outcome_identity(), recorded.outcome_identity());
            assert_eq!(original.operation_slot(), recorded.operation_slot());
        }
        (None, Some(_)) | (Some(_), None) => {
            panic!("retry must preserve the original declared outbox")
        }
    }
    match (live.aftermath_causality(), data.aftermath_causality()) {
        (None, None) => {}
        (Some(original), Some(recorded)) => {
            assert_eq!(original.role(), recorded.role());
            assert_eq!(original.parent(), recorded.parent());
            assert_eq!(original.child(), recorded.child());
            assert_eq!(original.fact_record(), recorded.fact_record());
            assert_eq!(&original.recorded_source().coordinates(), recorded.source());
        }
        (None, Some(_)) | (Some(_), None) => panic!("retry must preserve original causality"),
    }
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
