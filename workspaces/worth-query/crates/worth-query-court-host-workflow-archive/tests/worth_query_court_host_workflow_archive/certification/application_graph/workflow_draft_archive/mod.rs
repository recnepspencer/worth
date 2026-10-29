pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_draft_archive::*;

#[test]
fn operation_evidence_round_trips_and_cannot_silently_rebind_to_an_installed_floor() {
    let application = publish_workflow_on_first_program();
    let information =
        super::workflow_execution::definition::<OccurrenceSetRetentionBinding>().unwrap();
    let reviewed =
        reviewed_document_definition_for_binding::<ExplicitReviewedSetRetentionBinding>("done");
    for definition in [&information, &reviewed] {
        let bytes = encode(definition);
        let rebuilt = round_trip(&application, &bytes);
        assert_eq!(rebuilt.content_identity(), definition.content_identity());
        assert_eq!(encode(&rebuilt), bytes);
    }
    // A different explicit current floor remains untrusted metadata and cannot
    // reinterpret the installed binding. Omitted or displaced evidence is malformed.
    let bytes = encode(&information);
    let different = worth_query_host::facade::declaration::application_operation::ApplicationWorkflowExecutionRequirements::requiring_approval::<
        super::document_retention_model::workflow::WorkflowApprovalCapability,
    >().persistence_identity();
    assert_eq!(
        refusal(&application, &retext(&bytes, "2:occurrence", &different)),
        DraftDenial::ChangedMember
    );
    for replacement in ["", "1:none"] {
        let malformed = retext(&bytes, "2:occurrence", replacement);
        let denial =
            decode_workflow_definition_draft(&malformed, WorthQueryPackageArchiveLimits::DEFAULT)
                .expect_err("current draft requires an explicit current evidence identity");
        assert_eq!(denial.kind(), worth_query_package_archive::facade::WorthQueryPackageArchiveDenialKind::InvalidRecordShape);
    }
}

#[test]
fn every_definition_shape_round_trips_to_the_same_content_identity() {
    let application = publish_workflow_on_first_program();
    let component = component_definition(3)
        .validate()
        .expect("the component definition is valid");
    for definition in [
        reviewed_document_definition("applied"),
        approval_retry_definition(),
        condition_terminal_definition(),
        bounded_retry_definition(),
        conditionally_required_related_assessment_definition(),
        component,
    ] {
        let bytes = encode(&definition);
        assert_eq!(bytes, encode(&definition), "encoding is deterministic");
        let rebuilt = round_trip(&application, &bytes);
        assert_eq!(rebuilt.identity(), definition.identity());
        assert_eq!(
            rebuilt.content_identity(),
            definition.content_identity(),
            "{} keeps its content identity",
            definition.identity().as_str()
        );
        assert_eq!(
            encode(&rebuilt),
            bytes,
            "a rebuilt definition re-encodes exactly"
        );
    }
}

#[test]
fn a_rebuilt_draft_publishes_and_starts_like_the_typed_original() {
    let application = publish_workflow_on_first_program();
    let bytes = encode(&reviewed_document_definition("applied"));
    let rebuilt = round_trip(&application, &bytes);
    let published = match publish_definition(
        &application,
        rebuilt,
        WorkflowDefinitionExpectedPredecessor::Absent,
        701,
    )
    .expect("the rebuilt definition prepares for publication")
    {
        WorkflowDefinitionPublicationOutcome::Published(published) => {
            published.definition().clone()
        }
        other => panic!("the rebuilt definition must publish, got {other:?}"),
    };
    start_instance(&application, published, 702).expect("the published draft starts");
}

#[test]
fn drafts_naming_foreign_or_changed_vocabulary_are_refused_by_node() {
    let application = publish_workflow_on_first_program();
    let definition = reviewed_document_definition("applied");
    let bytes = encode(&definition);
    let (identifier, input_type) = definition
        .nodes()
        .iter()
        .find_map(|node| match node.kind() {
            ApplicationWorkflowNodeKind::Operation { operation, .. } => Some((
                operation.identifier().to_owned(),
                operation.input_type().as_str().to_owned(),
            )),
            _ => None,
        })
        .expect("the definition proposes through an operation");

    let foreign = retext(
        &bytes,
        <ReviewedDocumentWorkflow as worth_query_host::facade::declaration::application_program::ApplicationWorkflowSpec>::IDENTITY.as_str(),
        "worth.query.certification.foreign-workflow.v1",
    );
    assert_eq!(refusal(&application, &foreign), DraftDenial::ForeignSpec);

    let unknown = retext(&bytes, &identifier, "operation.not-installed");
    assert_eq!(refusal(&application, &unknown), DraftDenial::UnknownMember);

    let changed = retext(&bytes, &input_type, "changed.input.v1");
    assert_eq!(refusal(&application, &changed), DraftDenial::ChangedMember);
}

#[test]
fn drafts_naming_another_binding_or_capability_are_refused() {
    let application = publish_workflow_on_first_program();
    let definition = reviewed_document_definition("applied");
    let bytes = encode(&definition);
    let binding = definition
        .nodes()
        .iter()
        .find_map(|node| match node.kind() {
            ApplicationWorkflowNodeKind::Operation { operation, .. } => operation
                .binding()
                .map(|(identity, _, _)| identity.to_owned()),
            _ => None,
        })
        .expect("the definition performs through a bound operation");
    let (approval, capability) = definition
        .nodes()
        .iter()
        .find_map(|node| match node.kind() {
            ApplicationWorkflowNodeKind::Approval(approval) => Some((
                approval.identifier().to_owned(),
                approval.capability_type().as_str().to_owned(),
            )),
            _ => None,
        })
        .expect("the definition gates its effect on approval");

    let rebound = retext(&bytes, &binding, "binding.not-installed.v1");
    assert_eq!(refusal(&application, &rebound), DraftDenial::UnknownMember);

    // The capability follows its approval's identifier, which may share its
    // text, so only the field after the identifier is replaced.
    let widened = replace(
        &bytes,
        &[framed(&approval), framed(&capability)].concat(),
        &[framed(&approval), framed("changed.capability.v1")].concat(),
    );
    assert_eq!(refusal(&application, &widened), DraftDenial::ChangedMember);
}

#[test]
fn a_relation_the_schema_does_not_declare_is_refused() {
    let application = publish_workflow_on_first_program();
    let definition = conditionally_required_related_assessment_definition();
    let relation = definition
        .nodes()
        .iter()
        .find_map(|node| match node.kind() {
            ApplicationWorkflowNodeKind::Assessment(assessment) => assessment
                .applicability()
                .relation()
                .map(|(relation, _, _)| relation.to_owned()),
            _ => None,
        })
        .expect("the definition applies an assessment on a relation");
    let undeclared = retext(&encode(&definition), &relation, "UndeclaredRelation");
    assert_eq!(
        refusal(&application, &undeclared),
        DraftDenial::UndeclaredRelation
    );
}

#[test]
fn authoring_refuses_invalid_identities_and_validation_refuses_open_graphs() {
    let application = publish_workflow_on_first_program();
    let definition = reviewed_document_definition("applied");
    let bytes = encode(&definition);
    let start = definition.start().as_str();

    let invalid = restart(&bytes, start, "has space");
    assert_eq!(refusal(&application, &invalid), DraftDenial::Authoring);

    // A start naming no node authors, because authoring rebuilds meaning
    // only; the ordinary validator still refuses the open graph.
    let dangling = restart(&bytes, start, "missing-start");
    let authored = decode(&dangling)
        .author(application.workflow_spec())
        .unwrap_or_else(|denial| panic!("the dangling start still authors: {denial}"));
    assert!(
        authored.validate().is_err(),
        "an open graph never validates"
    );
}
