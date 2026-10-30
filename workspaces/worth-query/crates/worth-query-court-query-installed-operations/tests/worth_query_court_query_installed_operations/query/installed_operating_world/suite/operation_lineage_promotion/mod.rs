pub(crate) use worth_query_testkit_query::facade::query::suite::operation_lineage_promotion::*;

#[test]
fn promotion_mints_query_admitted_graph_identity_and_rejects_the_wrong_publication() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = lineage_workflow_workspace(
        request_execution,
        "lineage-promotion",
        domain::WorthQueryOperationLineageContract::Evolve,
        true,
        vec![LineageEvidenceScenario::SingularSuccessor],
    )
    .unwrap();
    let published = execute(&mut workspace).publish().unwrap();
    let expected_subject = published.trace().lineage_report().unwrap().evidence()[0]
        .outcome()
        .continuity_evidence()
        .unwrap()
        .successor_authoritative_identities()[0]
        .evidence_identity()
        .terminal_projection_for_reporting()
        .to_owned();
    let promoted = published
        .admit_promotion_on_reference(reference_intent("vertex"))
        .unwrap();
    assert_eq!(
        promoted
            .promoted_graph_identity()
            .carrying_artifact_identity()
            .as_str(),
        published.receipt_identity()
    );
    assert_eq!(
        promoted.promoted_graph_identity().subelement_key().as_str(),
        expected_subject
    );
    assert!(matches!(
        published.admit_promotion_on_reference(reference_intent("face")),
        TransitionOutcome::Denied(
            domain::WorthQueryPromotionOnReferenceDenial::CarryingPublicationMismatch
        )
    ));
    assert!(matches!(
        published.admit_promotion_on_reference(domain::WorthQueryDurableReferenceIntent::new(
            domain::WorthGraphDurableReferenceKind::PersistentSelection,
            domain::WorthQueryOperationProjectionRole::new("vertex").unwrap(),
            0,
            1,
        )),
        TransitionOutcome::Denied(
            domain::WorthQueryPromotionOnReferenceDenial::LineageSubjectMissing
        )
    ));
}

#[test]
fn plural_lineage_cannot_invent_authority_to_entity_correspondence_by_position() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = lineage_workflow_workspace(
        request_execution,
        "lineage-plural-promotion",
        domain::WorthQueryOperationLineageContract::Evolve,
        true,
        vec![LineageEvidenceScenario::SplitSuccessors],
    )
    .unwrap();
    let published = execute(&mut workspace).publish().unwrap();

    assert!(matches!(
        published.admit_promotion_on_reference(reference_intent("vertex")),
        TransitionOutcome::Denied(
            domain::WorthQueryPromotionOnReferenceDenial::LineageSubjectEntityBindingUnavailable
        )
    ));
}
