use crate::authoring::{
    AuthoredQueryBundleRequest, CollectionQueryBuilder, CollectionResultShapeBuilder,
    DetailQueryBuilder, DetailResultShapeBuilder, RawAuthoredQuery, RawAuthoredResultShape,
    RelationName, RootEntityKey, TraversalSelector,
};
use crate::binding::QueryBindingDescriptor;
use crate::canonicalization::canonicalize_request;
use crate::composition::ExpandedComposedIntent;
use crate::declarative_live::{validate_declared_traversal_contract, DeclarativeLiveQueryError};
use crate::ordinary::read::{
    WorthQueryDeclaredReadArtifacts, WorthQueryDeclaredReadIntent, WorthQueryDeclaredReadMeaning,
    WorthQueryDeclaredReadOperations, WorthQueryDeclaredTraversalContract,
    WorthQueryReadPlanningAuthority,
};
use crate::runtime::{
    QuerySchemaView, WorthQueryReadBuiltInOperator, WorthQueryReadDenial, WorthQueryReadDenialKind,
    WorthQueryReadGraph, WorthQueryReadGraphFamily, WorthQueryReadScopeClass,
};
use crate::validation::validate_canonical_bundle;

#[path = "read_composition_request.rs"]
mod request;

use super::read_composition_operator_builders::{
    CollectionReadOperatorQueryBuilder, DetailReadOperatorQueryBuilder,
};
use super::read_composition_relationship_proof::admit_read_relationship_proof;
use super::read_composition_runtime::classify_scope_shape_with_operators;
pub(in crate::runtime) use request::declarative_request_from_authored_shape;

pub(in crate::runtime) fn build_collection_read_intent(
    root: impl Into<String>,
    schema_view: QuerySchemaView,
    declare_query: impl FnOnce(CollectionQueryBuilder) -> CollectionQueryBuilder,
    declare_result_shape: impl FnOnce(CollectionResultShapeBuilder) -> CollectionResultShapeBuilder,
    expected_scope_class: WorthQueryReadScopeClass,
) -> Result<WorthQueryDeclaredReadIntent, WorthQueryReadDenial> {
    let (query, result_shape) =
        build_collection_authored_inputs(root, declare_query, declare_result_shape)?;
    build_scoped_read_intent_from_authored(
        query,
        result_shape,
        schema_view,
        WorthQueryReadGraphFamily::Collection,
        expected_scope_class,
        Vec::new(),
    )
}

pub(in crate::runtime) fn build_detail_read_intent(
    root: impl Into<String>,
    schema_view: QuerySchemaView,
    declare_query: impl FnOnce(DetailQueryBuilder) -> DetailQueryBuilder,
    declare_result_shape: impl FnOnce(DetailResultShapeBuilder) -> DetailResultShapeBuilder,
    expected_scope_class: WorthQueryReadScopeClass,
) -> Result<WorthQueryDeclaredReadIntent, WorthQueryReadDenial> {
    let (query, result_shape) =
        build_detail_authored_inputs(root, declare_query, declare_result_shape)?;
    build_scoped_read_intent_from_authored(
        query,
        result_shape,
        schema_view,
        WorthQueryReadGraphFamily::Detail,
        expected_scope_class,
        Vec::new(),
    )
}

pub(in crate::runtime) fn build_direct_edge_collection_read_intent(
    root: impl Into<String>,
    schema_view: QuerySchemaView,
    relation: RelationName,
    declare_query: impl FnOnce(CollectionReadOperatorQueryBuilder) -> CollectionReadOperatorQueryBuilder,
    declare_result_shape: impl FnOnce(CollectionResultShapeBuilder) -> CollectionResultShapeBuilder,
) -> Result<WorthQueryDeclaredReadIntent, WorthQueryReadDenial> {
    let (query, result_shape) =
        build_collection_operator_authored_inputs(root, declare_query, declare_result_shape)?;
    build_scoped_read_intent_from_authored(
        query.with_traversal(traversal_selector(relation, 1)?),
        result_shape,
        schema_view,
        WorthQueryReadGraphFamily::Collection,
        WorthQueryReadScopeClass::LocalNeighborhood,
        vec![WorthQueryReadBuiltInOperator::DirectEdge],
    )
}

pub(in crate::runtime) fn build_direct_edge_detail_read_intent(
    root: impl Into<String>,
    schema_view: QuerySchemaView,
    relation: RelationName,
    declare_query: impl FnOnce(DetailReadOperatorQueryBuilder) -> DetailReadOperatorQueryBuilder,
    declare_result_shape: impl FnOnce(DetailResultShapeBuilder) -> DetailResultShapeBuilder,
) -> Result<WorthQueryDeclaredReadIntent, WorthQueryReadDenial> {
    let (query, result_shape) =
        build_detail_operator_authored_inputs(root, declare_query, declare_result_shape)?;
    build_scoped_read_intent_from_authored(
        query.with_traversal(traversal_selector(relation, 1)?),
        result_shape,
        schema_view,
        WorthQueryReadGraphFamily::Detail,
        WorthQueryReadScopeClass::LocalNeighborhood,
        vec![WorthQueryReadBuiltInOperator::DirectEdge],
    )
}

fn build_collection_authored_inputs(
    root: impl Into<String>,
    declare_query: impl FnOnce(CollectionQueryBuilder) -> CollectionQueryBuilder,
    declare_result_shape: impl FnOnce(CollectionResultShapeBuilder) -> CollectionResultShapeBuilder,
) -> Result<(RawAuthoredQuery, RawAuthoredResultShape), WorthQueryReadDenial> {
    let root = parse_root(root)?;
    let query = declare_query(CollectionQueryBuilder::new(root))
        .build()
        .map_err(authoring_denial)?
        .into_raw();
    let result_shape = declare_result_shape(CollectionResultShapeBuilder::new())
        .build()
        .map_err(authoring_denial)?
        .into_raw();
    Ok((query, result_shape))
}

fn build_detail_authored_inputs(
    root: impl Into<String>,
    declare_query: impl FnOnce(DetailQueryBuilder) -> DetailQueryBuilder,
    declare_result_shape: impl FnOnce(DetailResultShapeBuilder) -> DetailResultShapeBuilder,
) -> Result<(RawAuthoredQuery, RawAuthoredResultShape), WorthQueryReadDenial> {
    let root = parse_root(root)?;
    let query = declare_query(DetailQueryBuilder::new(root))
        .build()
        .map_err(authoring_denial)?
        .into_raw();
    let result_shape = declare_result_shape(DetailResultShapeBuilder::new())
        .build()
        .map_err(authoring_denial)?
        .into_raw();
    Ok((query, result_shape))
}

pub(super) fn build_collection_operator_authored_inputs(
    root: impl Into<String>,
    declare_query: impl FnOnce(CollectionReadOperatorQueryBuilder) -> CollectionReadOperatorQueryBuilder,
    declare_result_shape: impl FnOnce(CollectionResultShapeBuilder) -> CollectionResultShapeBuilder,
) -> Result<(RawAuthoredQuery, RawAuthoredResultShape), WorthQueryReadDenial> {
    let root = parse_root(root)?;
    let query = declare_query(CollectionReadOperatorQueryBuilder::new(
        CollectionQueryBuilder::new(root),
    ))
    .finish()
    .build()
    .map_err(authoring_denial)?
    .into_raw();
    let result_shape = declare_result_shape(CollectionResultShapeBuilder::new())
        .build()
        .map_err(authoring_denial)?
        .into_raw();
    Ok((query, result_shape))
}

pub(super) fn build_detail_operator_authored_inputs(
    root: impl Into<String>,
    declare_query: impl FnOnce(DetailReadOperatorQueryBuilder) -> DetailReadOperatorQueryBuilder,
    declare_result_shape: impl FnOnce(DetailResultShapeBuilder) -> DetailResultShapeBuilder,
) -> Result<(RawAuthoredQuery, RawAuthoredResultShape), WorthQueryReadDenial> {
    let root = parse_root(root)?;
    let query = declare_query(DetailReadOperatorQueryBuilder::new(
        DetailQueryBuilder::new(root),
    ))
    .finish()
    .build()
    .map_err(authoring_denial)?
    .into_raw();
    let result_shape = declare_result_shape(DetailResultShapeBuilder::new())
        .build()
        .map_err(authoring_denial)?
        .into_raw();
    Ok((query, result_shape))
}

pub(super) fn build_scoped_read_intent_from_authored(
    query: RawAuthoredQuery,
    result_shape: RawAuthoredResultShape,
    schema_view: QuerySchemaView,
    family: WorthQueryReadGraphFamily,
    expected_scope_class: WorthQueryReadScopeClass,
    built_in_operators: Vec<WorthQueryReadBuiltInOperator>,
) -> Result<WorthQueryDeclaredReadIntent, WorthQueryReadDenial> {
    let request = AuthoredQueryBundleRequest::for_ordinary_read(
        query,
        result_shape,
        QueryBindingDescriptor::default(),
    )
    .map_err(authoring_denial)?;
    build_scoped_read_intent_from_request(
        request,
        schema_view,
        family,
        expected_scope_class,
        built_in_operators,
    )
}

pub(super) fn build_scoped_read_intent_from_composed(
    expanded: ExpandedComposedIntent,
    schema_view: QuerySchemaView,
    family: WorthQueryReadGraphFamily,
    expected_scope_class: WorthQueryReadScopeClass,
) -> Result<WorthQueryDeclaredReadIntent, WorthQueryReadDenial> {
    let (query, result_shape, bindings) = expanded.into_authored_request().into_parts();
    let authored = AuthoredQueryBundleRequest::for_ordinary_read(query, result_shape, bindings)
        .map_err(authoring_denial)?;
    build_scoped_read_intent_from_request(
        authored,
        schema_view,
        family,
        expected_scope_class,
        Vec::new(),
    )
}

fn build_scoped_read_intent_from_request(
    authored: AuthoredQueryBundleRequest,
    schema_view: QuerySchemaView,
    family: WorthQueryReadGraphFamily,
    expected_scope_class: WorthQueryReadScopeClass,
    built_in_operators: Vec<WorthQueryReadBuiltInOperator>,
) -> Result<WorthQueryDeclaredReadIntent, WorthQueryReadDenial> {
    let query = authored.query().clone();
    let result_shape = authored.result_shape().clone();
    let domain_graph_operations = query.domain_graph_operations().to_vec();
    let request =
        declarative_request_from_authored_shape(query, result_shape).map_err(declarative_denial)?;
    validate_declared_traversal_contract(&request, &schema_view).map_err(declarative_denial)?;
    let canonical = canonicalize_request(authored).map_err(canonicalization_denial)?;
    let schema_view_for_runtime = schema_view.clone();
    let validated =
        validate_canonical_bundle(canonical.clone(), schema_view).map_err(validation_denial)?;
    let scope_class = classify_scope_shape_with_operators(&validated, &built_in_operators);
    if scope_class != expected_scope_class {
        return Err(WorthQueryReadDenial::new_scope_shape_denied(
            expected_scope_class,
            scope_class,
        ));
    }
    let meaning = WorthQueryDeclaredReadMeaning {
        family,
        scope_class,
        schema_basis: validated.query().schema_basis().clone(),
    };
    let operations = WorthQueryDeclaredReadOperations {
        built_in: built_in_operators,
        domain: domain_graph_operations,
    };
    let traversal = WorthQueryDeclaredTraversalContract {
        clause_count: validated.query().traversal().len(),
        depth_limit: validated
            .query()
            .traversal()
            .iter()
            .map(|entry| usize::from(entry.depth()))
            .max()
            .unwrap_or(0),
    };
    let artifacts = WorthQueryDeclaredReadArtifacts {
        request,
        schema_view: schema_view_for_runtime,
        canonical,
        validated,
    };
    Ok(WorthQueryDeclaredReadIntent::new(
        meaning, operations, traversal, artifacts,
    )?)
}

pub(in crate::runtime) fn plan_standalone_read_intent(
    intent: WorthQueryDeclaredReadIntent,
) -> Result<WorthQueryReadGraph, WorthQueryReadDenial> {
    let relationship_proof_admission = admit_read_relationship_proof(
        intent.canonical().query(),
        intent.validated().query().schema_basis(),
        intent.validated().query().traversal(),
        intent.built_in_operators(),
    )?;
    intent.plan(WorthQueryReadPlanningAuthority::canonical(
        relationship_proof_admission,
    ))
}

fn parse_root(root: impl Into<String>) -> Result<RootEntityKey, WorthQueryReadDenial> {
    RootEntityKey::new(root.into()).map_err(|error| {
        WorthQueryReadDenial::new(WorthQueryReadDenialKind::InvalidRoot, format!("{error:?}"))
    })
}

pub(super) fn traversal_selector(
    relation: RelationName,
    max_depth: u8,
) -> Result<TraversalSelector, WorthQueryReadDenial> {
    TraversalSelector::bounded_relation_name(relation, max_depth).map_err(authoring_denial)
}

fn authoring_denial(error: impl std::fmt::Debug) -> WorthQueryReadDenial {
    WorthQueryReadDenial::new(
        WorthQueryReadDenialKind::AuthoringDenied,
        format!("{error:?}"),
    )
}

fn declarative_denial(error: DeclarativeLiveQueryError) -> WorthQueryReadDenial {
    let kind = match error {
        DeclarativeLiveQueryError::InvalidTarget
        | DeclarativeLiveQueryError::DuplicateTraversal { .. }
        | DeclarativeLiveQueryError::TraversalNotDeclaredInSchema { .. }
        | DeclarativeLiveQueryError::TraversalExceedsSchemaDepth { .. } => {
            WorthQueryReadDenialKind::ValidationDenied
        }
        DeclarativeLiveQueryError::Authoring(_) => WorthQueryReadDenialKind::AuthoringDenied,
        DeclarativeLiveQueryError::Canonicalization(_) => {
            WorthQueryReadDenialKind::CanonicalizationDenied
        }
        _ => WorthQueryReadDenialKind::PlanningDenied,
    };
    WorthQueryReadDenial::new(kind, format!("{error:?}"))
}

fn canonicalization_denial(error: impl std::fmt::Debug) -> WorthQueryReadDenial {
    WorthQueryReadDenial::new(
        WorthQueryReadDenialKind::CanonicalizationDenied,
        format!("{error:?}"),
    )
}

fn validation_denial(error: impl std::fmt::Debug) -> WorthQueryReadDenial {
    WorthQueryReadDenial::new(
        WorthQueryReadDenialKind::ValidationDenied,
        format!("{error:?}"),
    )
}
