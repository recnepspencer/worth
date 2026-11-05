//! No caller supplies content attachments to a genuine owner restoration.
use super::*;
use worth_relational::facade::{
    branch::*,
    content::RelationalContentDenial,
    mvcc::{PreparedRelationalCommitCandidate, RelationalPublicationOutcome},
};
fn publish(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    candidate: PreparedRelationalCommitCandidate,
) -> CommitResult {
    let services = runtime.owner_component_services();
    let RelationalPublicationOutcome::Performed(performed) = services
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("the actual owner candidate must publish")
    };
    services
        .settlement_port()
        .settle_performed_publication(performed)
        .unwrap()
}
fn entities(a: EntityId, x: EntityId, id: ContentRefId) -> Vec<RelationalEntityMaterialization> {
    [(a, KindId(1)), (x, KindId(4))]
        .map(|(entity_id, kind_id)| RelationalEntityMaterialization {
            entity_id,
            kind_id,
            fields: AspectFieldPatch::from_locator(
                planned_single_field_locator(registry::key(), registry::field("value")),
                AspectValue::ContentRef(id),
            ),
        })
        .to_vec()
}
fn relations(
    a: EntityId,
    x: EntityId,
    relation: RelationId,
) -> Vec<RelationalRelationMaterialization> {
    vec![RelationalRelationMaterialization {
        relation_id: relation,
        kind_id: KindId(2),
        source: a,
        target: x,
        fields: Default::default(),
    }]
}
#[test]
fn content_source_suspension_and_rematerialization_use_original_native_custody() {
    let (runtime, _program, court, _registration) = fixture();
    let cancel = CancellationSource::new();
    let host = crate::content::original(&cancel);
    let execution = ExecutionRequest::serial(&host);
    let (a, x, relation, id) = birth(&runtime, &execution);
    let original_checks = court.content_proposed_checks.load(Ordering::SeqCst);
    let services = runtime.owner_component_services();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let prepared = services
        .materialization_port()
        .prepare_generated_materialization_suspension(&execution, &basis, &[a, x])
        .unwrap();
    let (candidate, completion) = prepared.into_parts();
    let suspended = completion
        .complete(publish(&runtime, &execution, candidate))
        .unwrap();
    assert!(
        court.content_proposed_checks.load(Ordering::SeqCst) > original_checks,
        "logical Source still reads original frames while physical fields disappear"
    );
    let (_, unavailable) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert!(!unavailable.materialization_is_complete());
    assert_eq!(
        unavailable
            .read_content(&execution, id, CanonicalDigestId::new([11; 32]))
            .unwrap_err(),
        RelationalContentDenial::MissingContent
    );
    let prior = unavailable.descriptor().root_identity();
    let checks = court.content_checks.load(Ordering::SeqCst);
    let refused = services
        .materialization_port()
        .prepare_generated_rematerialization(
            &execution,
            &unavailable,
            suspended.custody,
            entities(a, x, ContentRefId([0xE7; 32])),
            relations(a, x, relation),
        );
    let failure = match refused {
        Err(cause) => cause,
        Ok(_) => panic!("missing content must refuse before Source preparation"),
    };
    assert!(
        matches!(
            failure.error,
            RelationalMaterializationError::Commit(TransactionCommitError::Conflict {
                error: CommitConflict {
                    class: ConflictClass::ContentAdmission {
                        denial: RelationalContentDenial::MissingContent
                    },
                    ..
                },
                ..
            })
        ),
        "{:?}",
        failure.error
    );
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor()
            .root_identity(),
        prior
    );
    assert_eq!(court.content_checks.load(Ordering::SeqCst), checks);
    let prepared = services
        .materialization_port()
        .prepare_generated_rematerialization(
            &execution,
            &unavailable,
            failure.custody,
            entities(a, x, id),
            relations(a, x, relation),
        )
        .unwrap();
    let (candidate, completion, _evidence) = prepared.into_parts();
    completion
        .complete(publish(&runtime, &execution, candidate))
        .unwrap();
    let (_, restored) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert!(restored.materialization_is_complete());
    assert_eq!(
        restored
            .read_content(&execution, id, CanonicalDigestId::new([11; 32]))
            .unwrap()
            .bytes(),
        crate::content::expected::FRAME
    );
    for (row, kind) in [(a, KindId(1)), (x, KindId(4))] {
        assert_eq!(
            restored
                .retain_field_content(&execution, row, kind, &registry::locator("value"), id)
                .unwrap()
                .bytes(),
            crate::content::expected::FRAME
        );
    }
}
