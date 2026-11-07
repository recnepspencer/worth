use worth_foundational::facade::{AspectKey, FieldKey};

use super::canonicalization::{normalized_query_projection, normalized_result_fields};
use super::*;
use crate::authoring::{
    AspectFieldKey, OrderingDirection, TraversalSelector, WorthQueryPredicateOperand,
};
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::schema_view::{QuerySchemaView, ScalarAspectType, SchemaFieldView, SchemaRelationView};
use crate::view_shape_live::LiveViewShapeFamily;
use crate::workflow::{WorkflowFreshnessBinding, WorkflowStalenessClass};

fn todo_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "todo-demo-schema",
        [
            SchemaFieldView::new(
                crate::authoring::AspectName::new("identity")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("id").expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                crate::authoring::AspectName::new("status")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("state")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                crate::authoring::AspectName::new("title")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("value")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
        ],
        [],
    )
}

fn test_snapshot_identity(label: &'static str) -> WorthQuerySnapshotIdentity {
    WorthQuerySnapshotIdentity::preview(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WriteReceiptSnapshotIdentity)
            .field_shape(WorthQueryEvidenceTag::new("test_snapshot"), label)
            .seal()
            .expect("fixture digest must fit"),
    )
}

fn test_field_key(aspect: &str, field: &str) -> AspectFieldKey {
    let aspect = AspectKey::new(aspect).expect("test aspect key should be valid");
    let field = FieldKey::new(field).expect("test field key should be valid");
    AspectFieldKey::from_native_keys(&aspect, &field)
}

#[test]
fn runtime_list_splice_declaration_mints_real_live_session_with_hidden_basis() {
    let request = DeclarativeLiveQueryRequest::new("Todo", DeclarativeLiveViewShape::list_splice())
        .where_equal(DeclarativeEqualityFilter::new(
            test_field_key("status", "state"),
            WorthQueryPredicateOperand::string("incomplete".to_string()),
        ));

    let snapshot_identity = test_snapshot_identity("runtime-head-demo");
    let session =
        declare_runtime_live_query_session(request, todo_schema(), snapshot_identity.clone())
            .expect("declarative list splice should plan, preflight, and lower to live");

    assert_eq!(session.request().target(), "Todo");
    assert_eq!(
        session.live_view().lowering().family(),
        LiveViewShapeFamily::Table
    );
    assert_eq!(
        session.preflight().basis().identity().snapshot_identity(),
        &snapshot_identity.evidence_identity()
    );
    assert_eq!(
        session.preflight().basis().identity().schema_basis(),
        session.view_plan().validated().query().schema_basis()
    );
    assert_eq!(
        session.preflight().basis().proof().digest(),
        session.live_view().basis().proof().digest()
    );
}

#[test]
fn projection_defaults_include_filter_and_ordering_fields_without_host_knobs() {
    let request = DeclarativeLiveQueryRequest::new("Todo", DeclarativeLiveViewShape::list_splice())
        .where_equal(DeclarativeEqualityFilter::new(
            test_field_key("status", "state"),
            WorthQueryPredicateOperand::string("incomplete".to_string()),
        ));

    let fields = normalized_query_projection(&request);

    assert_eq!(
        fields
            .iter()
            .map(|field| {
                (
                    field.source_field_key().aspect().as_str(),
                    field.source_field_key().field().as_str(),
                )
            })
            .collect::<Vec<_>>(),
        vec![("identity", "id"), ("status", "state")]
    );
}

#[test]
fn writeback_from_live_session_preserves_basis_and_detected_aspect_intent() {
    let session = declare_runtime_live_query_session(
        DeclarativeLiveQueryRequest::new("Todo", DeclarativeLiveViewShape::list_splice())
            .where_equal(DeclarativeEqualityFilter::new(
                test_field_key("status", "state"),
                WorthQueryPredicateOperand::string("incomplete".to_string()),
            )),
        todo_schema(),
        test_snapshot_identity("runtime-head-writeback"),
    )
    .expect("runtime live query should admit");

    let artifact = declare_writeback_from_live_session(
        &session,
        DeclarativeWritebackIntent::update_aspect(
            AspectFieldKey::from_authoring_parts("title", "value").unwrap(),
            DeclarativeWritebackValue::string("Buy oat milk"),
        ),
    )
    .expect("SDK-detected local proxy edit should lower to bridge writeback declaration");

    assert_eq!(artifact.changes().len(), 1);
    assert_eq!(
        artifact.changes()[0].source_field_key().aspect().as_str(),
        "title"
    );
    assert_eq!(
        artifact.changes()[0].source_field_key().field().as_str(),
        "value"
    );
    assert_eq!(
        artifact.live_view_basis_digest(),
        session.preflight().basis().proof().digest().as_str()
    );
    assert!(!artifact.intent_digest().is_empty());
    assert!(!artifact.artifact_digest().is_empty());
    assert_eq!(
        artifact.declaration().freshness_binding(),
        &WorkflowFreshnessBinding::RuntimeBasisExact
    );
    assert_eq!(
        artifact.declaration().staleness_class(),
        &WorkflowStalenessClass::AuthorityValidationRequired
    );
}

#[test]
fn empty_writeback_intent_is_rejected_before_bridge_lowering() {
    let session = declare_runtime_live_query_session(
        DeclarativeLiveQueryRequest::new("Todo", DeclarativeLiveViewShape::table()),
        todo_schema(),
        test_snapshot_identity("runtime-head-empty-writeback"),
    )
    .expect("runtime live query should admit");

    let error = declare_writeback_from_live_session(&session, DeclarativeWritebackIntent::new([]))
        .expect_err("empty proxy flushes should never mint writeback authority");

    assert_eq!(error, DeclarativeLiveQueryError::EmptyWritebackIntent);
}

#[test]
fn runtime_declarative_request_preserves_traversal_into_canonical_query() {
    let request = DeclarativeLiveQueryRequest::new("Todo", DeclarativeLiveViewShape::detail())
        .project(DeclarativeProjectionField::from_authoring_parts(
            "identity", "id",
        ))
        .traverse(TraversalSelector::bounded("worth.todo_parent", 2).unwrap());
    let schema = QuerySchemaView::new(
        "todo-demo-schema-with-traversal",
        [
            SchemaFieldView::new(
                crate::authoring::AspectName::new("identity")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("id").expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                crate::authoring::AspectName::new("status")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("state")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                crate::authoring::AspectName::new("title")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("value")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
        ],
        [SchemaRelationView::new(
            crate::authoring::RelationName::new("worth.todo_parent")
                .expect("schema relation literal must be valid"),
            2,
        )],
    );

    let session = declare_runtime_live_query_session(
        request,
        schema,
        test_snapshot_identity("runtime-head-traversal"),
    )
    .expect("declarative traversal should lower into the canonical query");

    assert_eq!(session.request().traversal().len(), 1);
    assert_eq!(session.canonical().query().traversal().len(), 1);
    assert_eq!(
        session.canonical().query().traversal()[0].relation.as_str(),
        "worth.todo_parent"
    );
    assert_eq!(session.canonical().query().traversal()[0].depth, 2);
}

#[test]
fn runtime_declarative_request_rejects_duplicate_traversal_before_canonicalization() {
    let request = DeclarativeLiveQueryRequest::new("Todo", DeclarativeLiveViewShape::detail())
        .project(DeclarativeProjectionField::from_authoring_parts(
            "identity", "id",
        ))
        .traverse(TraversalSelector::bounded("worth.todo_parent", 2).unwrap())
        .traverse(TraversalSelector::bounded("worth.todo_parent", 2).unwrap());
    let schema = QuerySchemaView::new(
        "todo-demo-schema-with-traversal",
        [
            SchemaFieldView::new(
                crate::authoring::AspectName::new("identity")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("id").expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                crate::authoring::AspectName::new("status")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("state")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                crate::authoring::AspectName::new("title")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("value")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
        ],
        [SchemaRelationView::new(
            crate::authoring::RelationName::new("worth.todo_parent")
                .expect("schema relation literal must be valid"),
            2,
        )],
    );

    let error = declare_runtime_live_query_session(
        request,
        schema,
        test_snapshot_identity("runtime-head-traversal"),
    )
    .expect_err("duplicate traversal should fail at the declarative boundary");

    assert!(matches!(
        error,
        DeclarativeLiveQueryError::DuplicateTraversal { .. }
    ));
}

#[test]
fn declarative_request_preserves_query_only_projection_and_delivered_result_fields() {
    let request = DeclarativeLiveQueryRequest::new("Todo", DeclarativeLiveViewShape::table())
        .project_query_only(DeclarativeProjectionField::from_authoring_parts(
            "identity", "id",
        ))
        .result_field(
            DeclarativeProjectionField::from_authoring_parts("title", "value")
                .delivered_as("title"),
        )
        .order_by_direction(DeclarativeOrderingField::descending(
            AspectFieldKey::from_authoring_parts("title", "value")
                .expect("test ordering key should be valid"),
        ));

    let query_projection = normalized_query_projection(&request);
    let result_fields = normalized_result_fields(&request, &query_projection);

    assert_eq!(
        query_projection
            .iter()
            .map(|field| {
                (
                    field.source_field_key().aspect().as_str(),
                    field.source_field_key().field().as_str(),
                )
            })
            .collect::<Vec<_>>(),
        vec![("identity", "id"), ("title", "value")]
    );
    assert_eq!(
        result_fields
            .iter()
            .map(|field| {
                (
                    field.source_field_key().aspect().as_str(),
                    field.source_field_key().field().as_str(),
                    field.delivered_name(),
                )
            })
            .collect::<Vec<_>>(),
        vec![("title", "value", "title")]
    );
}

#[test]
fn runtime_declarative_request_preserves_non_equality_predicates_and_descending_ordering() {
    let request = DeclarativeLiveQueryRequest::new("Todo", DeclarativeLiveViewShape::table())
        .project(DeclarativeProjectionField::from_authoring_parts(
            "identity", "id",
        ))
        .where_greater_than(DeclarativeNativeComparisonFilter::greater_than(
            test_field_key("metrics", "priority"),
            5,
        ))
        .where_contains(DeclarativeStringContainsFilter::new(
            test_field_key("title", "value"),
            "milk",
        ))
        .where_in(DeclarativeSetMembershipFilter::new(
            test_field_key("status", "state"),
            [
                WorthQueryPredicateOperand::string("todo".to_string()),
                WorthQueryPredicateOperand::string("doing".to_string()),
            ],
        ))
        .where_present(DeclarativePresenceFilter::is_present(test_field_key(
            "owner", "name",
        )))
        .order_by_direction(DeclarativeOrderingField::descending(test_field_key(
            "metrics", "priority",
        )));

    let canonical = canonicalize_declarative_request(&request)
        .expect("declarative request should preserve full predicate families");

    assert_eq!(canonical.query().predicates().len(), 4);
    assert_eq!(canonical.query().ordering().len(), 1);
    assert_eq!(
        canonical.query().ordering()[0].direction,
        OrderingDirection::Descending
    );
}
