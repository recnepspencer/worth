use crate::declarative_live::canonicalize_declarative_request;
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::projection_consumption::ProjectionMaterializedFactPosture;
use crate::query_context::{
    execute_query_basis_context, QueryContextFamily, ScopedQueryBasisContext,
};
use crate::runtime::{
    WorthQueryCountResult, WorthQueryEphemeralGraphIndexReceipt,
    WorthQueryGraphReadAccessExecutionCounters, WorthQueryReadBuiltInOperator,
    WorthQueryReadDenial, WorthQueryReadDenialKind, WorthQueryReadExecutionProduct,
    WorthQueryReadGraph, WorthQueryReadResult, WorthQueryReadScopeClass, WorthQueryRuntime,
};
use worth_foundational::facade::{AspectKey, FieldKey};

use super::materialized_fact_posture::materialized_fact_posture_from_live_subscription_state;
use super::read_composition_materialization::{
    materialize_query_context_rows, materialize_read_rows,
};

pub(in crate::runtime) struct WorthQueryExecutedReadProduct<Product> {
    pub(super) product: Product,
    pub(super) graph_read_access_execution_counters: WorthQueryGraphReadAccessExecutionCounters,
}

impl<Product> WorthQueryExecutedReadProduct<Product>
where
    Product: WorthQueryReadExecutionProduct,
{
    pub(in crate::runtime) fn product(&self) -> &Product {
        &self.product
    }

    pub(in crate::runtime) fn product_mut(&mut self) -> &mut Product {
        &mut self.product
    }

    pub(in crate::runtime) fn graph_read_access_execution_counters(
        &self,
    ) -> &WorthQueryGraphReadAccessExecutionCounters {
        &self.graph_read_access_execution_counters
    }

    pub(in crate::runtime) fn record_ephemeral_index_receipt(
        &mut self,
        receipt: Option<&WorthQueryEphemeralGraphIndexReceipt>,
    ) {
        let Some(receipt) = receipt else {
            return;
        };
        self.graph_read_access_execution_counters
            .record_ephemeral_index_allocations(receipt.counters().successful_allocation_count());
    }

    pub(in crate::runtime) fn into_product(self) -> Product {
        self.product
    }
}

pub(in crate::runtime) type WorthQueryExecutedReadGraph =
    WorthQueryExecutedReadProduct<WorthQueryReadResult>;
pub(in crate::runtime) type WorthQueryExecutedCountGraph =
    WorthQueryExecutedReadProduct<WorthQueryCountResult>;

pub(super) fn classify_scope_shape_with_operators(
    validated: &crate::validation::ValidatedQueryBundle,
    built_in_operators: &[WorthQueryReadBuiltInOperator],
) -> WorthQueryReadScopeClass {
    let traversal = validated.query().traversal();
    let traversal_depth_limit = traversal
        .iter()
        .map(|entry| entry.depth())
        .max()
        .unwrap_or(0);
    let non_anchor_predicate_count = validated
        .query()
        .predicates()
        .entries()
        .iter()
        .filter(|predicate| !is_identity_anchor_predicate(predicate))
        .count();

    if built_in_operators.contains(&WorthQueryReadBuiltInOperator::FrontierSearch)
        || non_anchor_predicate_count > 0
    {
        WorthQueryReadScopeClass::ExplicitBroadSearch
    } else if built_in_operators.contains(&WorthQueryReadBuiltInOperator::SuccessorWalk)
        || built_in_operators.contains(&WorthQueryReadBuiltInOperator::DirectEdge)
        || built_in_operators.contains(&WorthQueryReadBuiltInOperator::SharedEndpoint)
        || built_in_operators.contains(&WorthQueryReadBuiltInOperator::SharedAttachment)
    {
        WorthQueryReadScopeClass::LocalNeighborhood
    } else if traversal_depth_limit > 1 {
        WorthQueryReadScopeClass::AnchoredExpansion
    } else {
        WorthQueryReadScopeClass::LocalNeighborhood
    }
}

fn is_identity_anchor_predicate(predicate: &crate::validation::ValidatedPredicateEntry) -> bool {
    let anchor = NativeIdentityAnchorPredicateKey::new();
    predicate.native_aspect_key() == anchor.native_aspect_key()
        && predicate.native_field_key() == anchor.native_field_key()
        && predicate.predicate_family() == "equality"
        && predicate.value_kind() == "String"
}

struct NativeIdentityAnchorPredicateKey {
    aspect_key: AspectKey,
    field_key: FieldKey,
}

impl NativeIdentityAnchorPredicateKey {
    fn new() -> Self {
        Self {
            aspect_key: AspectKey::new("identity")
                .expect("identity anchor aspect key should be foundational"),
            field_key: FieldKey::new("id")
                .expect("identity anchor field key should be foundational"),
        }
    }

    fn native_aspect_key(&self) -> &AspectKey {
        &self.aspect_key
    }

    fn native_field_key(&self) -> &FieldKey {
        &self.field_key
    }
}

pub(in crate::runtime) fn execute_runtime_basis_context_read_graph(
    runtime: &mut WorthQueryRuntime,
    read_graph: &WorthQueryReadGraph,
    context: &ScopedQueryBasisContext,
) -> Result<WorthQueryExecutedReadGraph, WorthQueryReadDenial> {
    ensure_context_matches_read_graph(read_graph, context)?;
    let context_execution = execute_query_basis_context(context).map_err(|error| {
        WorthQueryReadDenial::new(
            WorthQueryReadDenialKind::BasisPreflightDenied,
            error.message().to_string(),
        )
    })?;
    let context_execution =
        context_execution.with_materialized_fact_posture(materialized_fact_posture_for_read_graph(
            runtime,
            read_graph,
            &query_context_basis_digest_identity(context.basis_digest()),
        ));
    let receipt_snapshot_identity = runtime.current_snapshot_identity()?;
    let (rows, maintenance_source_rows) =
        if context_allows_runtime_materialization(&receipt_snapshot_identity, context) {
            let (rows, support_rows, _) = materialize_read_rows(runtime, read_graph)?.into_parts();
            (rows, Some(support_rows))
        } else {
            (materialize_query_context_rows(&context_execution), None)
        };
    let graph_read_access_execution_counters =
        WorthQueryGraphReadAccessExecutionCounters::observed_admitted_execution(rows.len());
    let receipt = crate::runtime::WorthQueryReadReceipt::from_query_context_execution(
        read_graph,
        receipt_snapshot_identity,
        &context_execution,
        &rows,
    );
    let product = match maintenance_source_rows {
        Some(source_rows) => WorthQueryReadResult::new_with_source_rows(rows, source_rows, receipt),
        None => WorthQueryReadResult::new(rows, receipt),
    };
    Ok(WorthQueryExecutedReadProduct {
        graph_read_access_execution_counters,
        product,
    })
}

pub(super) fn materialized_fact_posture_for_read_graph(
    runtime: &WorthQueryRuntime,
    read_graph: &WorthQueryReadGraph,
    basis_identity: &crate::WorthQueryEvidenceIdentity,
) -> Option<ProjectionMaterializedFactPosture> {
    let lower_declaration_digest =
        canonicalize_declarative_request(read_graph.declarative_request())
            .ok()?
            .query()
            .digest()
            .as_str()
            .to_string();
    let mut exact_request_matches = runtime
        .live_subscriptions
        .values()
        .filter(|state| state.request == *read_graph.declarative_request());
    let state = if let Some(state) = exact_request_matches.next() {
        if exact_request_matches.next().is_some() {
            return None;
        }
        state
    } else {
        let mut canonical_matches = runtime.live_subscriptions.values().filter(|state| {
            state.installation.query_projection().label() == lower_declaration_digest.as_str()
        });
        let state = canonical_matches.next()?;
        if canonical_matches.next().is_some() {
            return None;
        }
        state
    };
    Some(materialized_fact_posture_from_live_subscription_state(
        state,
        basis_identity,
    ))
}

fn query_context_basis_digest_identity(basis_digest: &str) -> crate::WorthQueryEvidenceIdentity {
    crate::WorthQueryEvidenceIdentity::compose(
        crate::WorthQueryEvidenceScope::QueryContextCompatibilityBasisLabel,
    )
    .field_value(
        crate::WorthQueryEvidenceTag::new("basis_digest"),
        basis_digest,
    )
    .seal()
}

fn ensure_context_matches_read_graph(
    read_graph: &WorthQueryReadGraph,
    context: &ScopedQueryBasisContext,
) -> Result<(), WorthQueryReadDenial> {
    if context.query_digest() == read_graph.query_digest() {
        return Ok(());
    }
    Err(WorthQueryReadDenial::new(
        WorthQueryReadDenialKind::BasisPreflightDenied,
        "admitted query basis context does not match reusable read-family query digest",
    ))
}

fn context_allows_runtime_materialization(
    runtime_snapshot_identity: &WorthQuerySnapshotIdentity,
    context: &ScopedQueryBasisContext,
) -> bool {
    match context.family() {
        QueryContextFamily::CurrentBranchHead => true,
        QueryContextFamily::HistoricalSnapshot => {
            context.admits_runtime_snapshot(runtime_snapshot_identity)
        }
        QueryContextFamily::BranchHead
        | QueryContextFamily::HistoricalCommit
        | QueryContextFamily::PreviewDerivedHistorical
        | QueryContextFamily::DiffComparison => false,
    }
}
