use super::super::support::*;
use crate::authoring::{
    AspectFieldSelector, AuthoredResultShapeField, CollectionResultShapeBuilder,
    DetailResultShapeBuilder, GuidedAuthoringPath, OrderingSelector, RawAuthoredQuery,
    RootEntityKey, StringContainsPredicate, TraversalSelector,
};
use crate::declarative_live::canonicalize_declarative_request;
use crate::runtime::read_composition_lowering::declarative_request_from_authored_shape;
use crate::runtime::WorthQueryReadScopeClass;
use crate::validation::validate_canonical_bundle;

#[test]
fn compose_read_detail_matches_declarative_request_canonical_query_for_hidden_query_projection() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.declarative-detail")
        .expect("read-backed runtime should open a workspace");

    let result = workspace
        .compose_read(request_execution, |read| {
            read.local_detail(
                "user",
                manager_schema(),
                |query| {
                    query
                        .project(
                            AspectFieldSelector::new("identity", "id")
                                .expect("identity projection should build"),
                        )
                        .project(
                            AspectFieldSelector::new("profile", "display_name")
                                .expect("name projection should build"),
                        )
                        .traverse(
                            TraversalSelector::bounded("manager", 1)
                                .expect("bounded traversal should build"),
                        )
                },
                |shape| {
                    shape.field(
                        AuthoredResultShapeField::new("identity", "id", "user_id")
                            .expect("identity result-shape field should build"),
                    )
                },
            )
        })
        .expect("local detail read should execute");

    let root = RootEntityKey::new("user").expect("detail root should build");
    let query = RawAuthoredQuery::detail_builder(root)
        .project(AspectFieldSelector::new("identity", "id").expect("identity projection"))
        .project(AspectFieldSelector::new("profile", "display_name").expect("name projection"))
        .traverse(TraversalSelector::bounded("manager", 1).expect("bounded traversal"))
        .build()
        .expect("detail query should build")
        .into_raw();
    let result_shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("identity", "id", "user_id").expect("shape field"))
        .build()
        .expect("detail result shape should build")
        .into_raw();
    let request = declarative_request_from_authored_shape(query.clone(), result_shape.clone())
        .expect("declarative request should derive from authored detail shape");
    let canonical_from_request =
        canonicalize_declarative_request(&request).expect("request canonicalization should work");
    let validated_from_request =
        validate_canonical_bundle(canonical_from_request.clone(), manager_schema())
            .expect("request canonicalization should validate");
    let direct_root = RootEntityKey::new("user").expect("detail root should build");
    let direct_query = RawAuthoredQuery::detail_builder(direct_root)
        .project(AspectFieldSelector::new("identity", "id").expect("identity projection"))
        .project(AspectFieldSelector::new("profile", "display_name").expect("name projection"))
        .traverse(TraversalSelector::bounded("manager", 1).expect("bounded traversal"))
        .build()
        .expect("detail query should build");
    let direct_result_shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("identity", "id", "user_id").expect("shape field"))
        .build()
        .expect("detail result shape should build");
    let canonical_direct =
        GuidedAuthoringPath::canonicalize_detail(direct_query, direct_result_shape)
            .expect("direct parity");

    assert_eq!(
        result.receipt().query_digest(),
        validated_from_request.query().digest().as_str()
    );
    assert_eq!(
        canonical_direct.query().digest().as_str(),
        canonical_from_request.query().digest().as_str()
    );
    assert_eq!(
        request.query_projection().len(),
        2,
        "hidden query projection should remain separate from delivered fields"
    );
    assert_eq!(request.result_fields().len(), 1);
    assert_eq!(
        result.receipt().scope_class(),
        &WorthQueryReadScopeClass::LocalNeighborhood
    );
    assert_descriptor_admitted_synthetic_runtime_relationship_proof(
        &result,
        2,
        &direct_edge_synthetic_runtime_surfaces(),
    );
}

#[test]
fn compose_read_collection_matches_declarative_request_for_non_equality_predicate_and_descending_ordering(
) {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.declarative-collection")
        .expect("read-backed runtime should open a workspace");

    let result = workspace
        .compose_read(request_execution, |read| {
            read.explicit_broad_search_collection(
                "user",
                searchable_manager_schema(),
                |query| {
                    query
                        .project(
                            AspectFieldSelector::new("identity", "id")
                                .expect("identity projection should build"),
                        )
                        .project(
                            AspectFieldSelector::new("profile", "display_name")
                                .expect("name projection should build"),
                        )
                        .where_contains(
                            StringContainsPredicate::new("profile", "display_name", "Ada")
                                .expect("contains predicate should build"),
                        )
                        .order_by(
                            OrderingSelector::descending("profile", "display_name")
                                .expect("descending ordering should build"),
                        )
                },
                |shape| {
                    shape.field(
                        AuthoredResultShapeField::new("identity", "id", "id")
                            .expect("identity result-shape field should build"),
                    )
                },
            )
        })
        .expect("broad-search collection read should execute");

    let root = RootEntityKey::new("user").expect("collection root should build");
    let query = RawAuthoredQuery::collection_builder(root)
        .project(AspectFieldSelector::new("identity", "id").expect("identity projection"))
        .project(AspectFieldSelector::new("profile", "display_name").expect("name projection"))
        .where_contains(
            StringContainsPredicate::new("profile", "display_name", "Ada")
                .expect("contains predicate should build"),
        )
        .order_by(
            OrderingSelector::descending("profile", "display_name")
                .expect("descending ordering should build"),
        )
        .build()
        .expect("collection query should build")
        .into_raw();
    let result_shape = CollectionResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("identity", "id", "id").expect("shape field"))
        .build()
        .expect("collection result shape should build")
        .into_raw();
    let request = declarative_request_from_authored_shape(query.clone(), result_shape.clone())
        .expect("declarative request should derive from authored collection shape");
    let canonical_from_request =
        canonicalize_declarative_request(&request).expect("request canonicalization should work");
    let validated_from_request =
        validate_canonical_bundle(canonical_from_request.clone(), searchable_manager_schema())
            .expect("request canonicalization should validate");
    let direct_root = RootEntityKey::new("user").expect("collection root should build");
    let direct_query = RawAuthoredQuery::collection_builder(direct_root)
        .project(AspectFieldSelector::new("identity", "id").expect("identity projection"))
        .project(AspectFieldSelector::new("profile", "display_name").expect("name projection"))
        .where_contains(
            StringContainsPredicate::new("profile", "display_name", "Ada")
                .expect("contains predicate should build"),
        )
        .order_by(
            OrderingSelector::descending("profile", "display_name")
                .expect("descending ordering should build"),
        )
        .build()
        .expect("collection query should build");
    let direct_result_shape = CollectionResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("identity", "id", "id").expect("shape field"))
        .build()
        .expect("collection result shape should build");
    let canonical_direct =
        GuidedAuthoringPath::canonicalize_collection(direct_query, direct_result_shape)
            .expect("direct parity");

    assert_eq!(
        result.receipt().query_digest(),
        validated_from_request.query().digest().as_str()
    );
    assert_eq!(
        canonical_direct.query().digest().as_str(),
        canonical_from_request.query().digest().as_str()
    );
    assert_eq!(request.result_fields().len(), 1);
    assert_eq!(request.ordering().len(), 1);
    assert_eq!(
        result.receipt().scope_class(),
        &WorthQueryReadScopeClass::ExplicitBroadSearch
    );
    assert_relationship_proof_not_required(&result);
}
