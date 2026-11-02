//! Assert original recorded meaning against independently held fresh custody.
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryRecoveredApplicationCommitReceipt,
};
pub(in crate::domain_computation::primary_graph) fn assert_original(
    recorded: &WorthQueryRecoveredApplicationCommitReceipt,
    fresh: &WorthQueryApplicationCommitReceipt,
) {
    assert_eq!(recorded.commit_reference(), fresh.commit_reference());
    assert_eq!(
        recorded.changed_record_count(),
        fresh.changed_record_count()
    );
    assert_eq!(Some(recorded.outcome_identity()), fresh.outcome_identity());
    assert_eq!(
        recorded.emitted_effect_count(),
        fresh.emitted_effect_count()
    );
    assert_eq!(
        recorded.exact_output_settlement(),
        fresh.recorded_output_settlement()
    );
    let publication = fresh.committed_product_publication();
    let head = recorded
        .new_product_head()
        .memory()
        .expect("the fixture is explicitly memory owned");
    assert_eq!(
        head.owner(),
        publication.product_branch().owner_identity().get()
    );
    assert_eq!(head.branch(), publication.product_branch().name().as_str());
    assert_eq!(
        head.incarnation(),
        publication.product_incarnation().ordinal()
    );
    assert_eq!(head.generation(), publication.product_generation().get());
    assert_eq!(
        head.selected_commit(),
        publication.composite_commit().ordinal()
    );
    assert_eq!(
        recorded.attempt_identity().ordinal(),
        publication.publication_attempt().ordinal()
    );
    match (recorded.dispatch_outbox(), fresh.dispatch_outbox()) {
        (None, None) => {}
        (Some(recorded), Some(fresh)) => {
            assert_eq!(recorded.correlation(), fresh.correlation());
            assert_eq!(
                recorded.correlation_family(),
                fresh.correlation_family().as_str()
            );
            assert_eq!(recorded.effect(), fresh.effect());
            assert_eq!(recorded.protocol_identity(), fresh.protocol_identity());
            assert_eq!(recorded.protocol_version(), fresh.protocol_version());
            assert_eq!(
                recorded.maximum_payload_bytes(),
                fresh.maximum_payload_bytes()
            );
            assert_eq!(recorded.payload(), fresh.payload());
            assert_eq!(recorded.outcome_identity(), fresh.outcome_identity());
            assert_eq!(recorded.operation_slot(), fresh.operation_slot());
        }
        shapes => panic!("original outbox presence differs: {shapes:?}"),
    }
    match (recorded.aftermath_causality(), fresh.aftermath_causality()) {
        (None, None) => {}
        (Some(recorded), Some(fresh)) => assert_causality(recorded, fresh),
        shapes => panic!("original causality presence differs: {shapes:?}"),
    }
    assert!(recorded.publishing_incarnation().is_none());
    assert!(recorded.world_journal_position().is_none());
    assert!(recorded.world_artifact_digest().is_none());
}

pub(in crate::domain_computation::primary_graph) fn read_original(
    world: &super::AuthorizationWorld,
    fresh: &WorthQueryApplicationCommitReceipt,
) -> WorthQueryRecoveredApplicationCommitReceipt {
    world.application.with_host_advancement(|phase| {
        let product = world.selected_product(&phase.test_host_request()).product().publication_binding();
        match world.application.primary_provider.resolve_idempotency_binding_at_product(&phase.borrowed_request_for_test(), fresh.idempotency_binding(), &product).unwrap() {
            crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolution::Readmitted(original) => original,
            other => panic!("actual owner lookup must return original meaning: {other:?}"),
        }
    }).unwrap()
}

pub(in crate::domain_computation::primary_graph) fn assert_causality(
    recorded: &crate::domain_computation::primary_graph::WorthQueryRecoveredAftermathCausality,
    fresh: &crate::domain_computation::application_aftermath::WorthQueryCommittedAftermathCausality,
) {
    assert_eq!(recorded.role(), fresh.role());
    assert_eq!(recorded.parent(), fresh.parent());
    assert_eq!(recorded.child(), fresh.child());
    assert_eq!(recorded.fact_record(), fresh.fact_record());
    assert_eq!(recorded.source(), &fresh.recorded_source().coordinates());
    let lineage = fresh.correction_lineage();
    assert_eq!(recorded.root(), &lineage.root().coordinates());
    assert_eq!(
        recorded.selected_alternative(),
        lineage
            .selected_alternative()
            .map(|source| source.coordinates())
            .as_ref()
    );
    assert_eq!(
        recorded.active_reversal(),
        lineage
            .active_reversal()
            .map(|source| source.coordinates())
            .as_ref()
    );
    assert_eq!(
        recorded.prior_transition(),
        lineage
            .prior_transition()
            .map(|source| source.coordinates())
            .as_ref()
    );
}
