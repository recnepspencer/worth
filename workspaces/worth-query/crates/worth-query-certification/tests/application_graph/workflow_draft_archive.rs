//! Public certification that an authored definition travels as a bounded,
//! authority-free draft and authors again only against installed vocabulary.

use worth_query_host::facade::{
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    },
    declaration::application_program::{ApplicationWorkflowNodeKind, ValidatedWorkflowDefinition},
};
use worth_query_package_archive::facade::{
    decode_workflow_definition_draft, encode_workflow_definition_draft,
    WorthQueryPackageArchiveLimits, WorthQueryUntrustedWorkflowDefinitionDraft,
    WorthQueryWorkflowDefinitionDraftDenialKind as DraftDenial,
};

use super::bounded_dimension_model::{
    host::{publish_workflow_on_first_program, BoundedDimensionWorkflowRuntime},
    workflow::{
        approval_retry_definition, bounded_retry_definition, condition_terminal_definition,
        conditionally_required_related_assessment_definition, publish_definition,
        reviewed_geometry_definition, start_instance, ReviewedGeometryWorkflow,
    },
};
use super::workflow_component_scale::component_definition;

type Definition = ValidatedWorkflowDefinition<ReviewedGeometryWorkflow>;

#[test]
fn every_definition_shape_round_trips_to_the_same_content_identity() {
    let application = publish_workflow_on_first_program();
    let component = component_definition(3)
        .validate()
        .expect("the component definition is valid");
    for definition in [
        reviewed_geometry_definition("applied"),
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
    let bytes = encode(&reviewed_geometry_definition("applied"));
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
    let definition = reviewed_geometry_definition("applied");
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
        <ReviewedGeometryWorkflow as worth_query_host::facade::declaration::application_program::ApplicationWorkflowSpec>::IDENTITY.as_str(),
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
    let definition = reviewed_geometry_definition("applied");
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
    let definition = reviewed_geometry_definition("applied");
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

fn encode(definition: &Definition) -> Vec<u8> {
    encode_workflow_definition_draft(definition, WorthQueryPackageArchiveLimits::DEFAULT)
        .expect("the definition fits the archive budget")
}

fn decode(bytes: &[u8]) -> WorthQueryUntrustedWorkflowDefinitionDraft {
    decode_workflow_definition_draft(bytes, WorthQueryPackageArchiveLimits::DEFAULT)
        .expect("the draft bytes are well formed")
}

fn round_trip(application: &BoundedDimensionWorkflowRuntime, bytes: &[u8]) -> Definition {
    decode(bytes)
        .author(application.workflow_spec())
        .expect("the draft authors against installed vocabulary")
        .validate()
        .expect("the authored draft is valid")
}

fn refusal(application: &BoundedDimensionWorkflowRuntime, bytes: &[u8]) -> DraftDenial {
    match decode(bytes).author(application.workflow_spec()) {
        Ok(_) => panic!("the draft must not author against installed vocabulary"),
        Err(denial) => denial.kind(),
    }
}

/// Replaces only the header's start, which precedes every node record.
fn restart(bytes: &[u8], start: &str, replacement: &str) -> Vec<u8> {
    let marker = framed(start);
    let position = bytes
        .windows(marker.len())
        .position(|window| window == marker.as_slice())
        .expect("the header names the start");
    let mut output = bytes[..position].to_vec();
    output.extend_from_slice(&framed(replacement));
    output.extend_from_slice(&bytes[position + marker.len()..]);
    output
}

fn framed(text: &str) -> Vec<u8> {
    let length = u32::try_from(text.len()).expect("test text is short");
    let mut bytes = length.to_be_bytes().to_vec();
    bytes.extend_from_slice(text.as_bytes());
    bytes
}

/// Replaces every length-framed occurrence of `from` with `to`.
fn retext(bytes: &[u8], from: &str, to: &str) -> Vec<u8> {
    replace(bytes, &framed(from), &framed(to))
}

/// Replaces every occurrence of the byte sequence `from` with `to`.
fn replace(bytes: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    let mut replaced = false;
    while index < bytes.len() {
        if bytes[index..].starts_with(from) {
            output.extend_from_slice(to);
            index += from.len();
            replaced = true;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    assert!(replaced, "the draft carries the text being replaced");
    output
}
