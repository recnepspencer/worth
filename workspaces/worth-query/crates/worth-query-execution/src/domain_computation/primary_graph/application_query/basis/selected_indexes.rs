//! Index currency for the exact installed read that producer readmission will execute.

use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::indexes::DerivedIndexId;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::super::{
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
};
use crate::domain_computation::primary_graph::index_currency::{
    ensure_selected_field_indexes_admitted, SelectedFieldIndexAdmissionStop,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraph;

pub(in crate::domain_computation::primary_graph) use crate::domain_computation::primary_graph::selected_read_indexes::PreparedSelectedReadIndexes;

pub(in crate::domain_computation::primary_graph::application_query) fn ensure_selected_read_indexes_admitted<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Scope,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &WorthQueryPrimaryGraph,
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    basis: &AdmittedRelationalBranchBasis,
    admission: &mut InvalidationEditAdmission,
) -> Result<PreparedSelectedReadIndexes, WorthQueryApplicationQueryAdmissionDenial> {
    // Installed contract, root-selection mode, and optional live-target visits.
    admission
        .charge_external_work(3)
        .map_err(|_| work_denial(query.name()))?;
    let contract = query.read_family_binding().planning_contract();
    let root = if contract.root_paths().is_empty() {
        match contract.predicates() {
            [] => None,
            [predicate] => Some(selected_field_id(graph, predicate.field(), admission)?),
            _ => return Err(denial(query.name())),
        }
    } else {
        None
    };
    let target = query
        .live()
        .map(|live| {
            let field = live.target_identity();
            selected_field_id(
                graph,
                (field.entity(), field.aspect(), field.field()),
                admission,
            )
        })
        .transpose()?;
    // One Arc pin plus the two permitted owner-pointer/atomic checks: initial
    // Query authorization and its one-shot read. Later reuse fails closed.
    admission
        .charge_external_work(5)
        .map_err(|_| work_denial(query.name()))?;
    let retained_basis = basis.clone();
    graph
        .with_runtime(execution, |runtime| {
            ensure_selected_field_indexes_admitted(execution, runtime, basis, [root, target], admission)
        })
        .map_err(|stop| match stop {
            SelectedFieldIndexAdmissionStop::Admission(
                CompanionPreflightStop::PreparationMemoryExhausted { .. }
                | CompanionPreflightStop::PreparationMemoryCounterOverflow,
            ) => WorthQueryApplicationQueryAdmissionDenial::new(
                WorthQueryApplicationQueryAdmissionDenialKind::ReadmissionPreparationMemoryExhausted,
                query.name(),
            ),
            SelectedFieldIndexAdmissionStop::Admission(_) => WorthQueryApplicationQueryAdmissionDenial::new(
                WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded,
                query.name(),
            ),
            SelectedFieldIndexAdmissionStop::Native => denial(query.name()),
        })?;
    Ok(PreparedSelectedReadIndexes::for_query(retained_basis))
}

fn selected_field_id(
    graph: &WorthQueryPrimaryGraph,
    (entity, aspect, field): (&str, &str, &str),
    admission: &mut InvalidationEditAdmission,
) -> Result<DerivedIndexId, WorthQueryApplicationQueryAdmissionDenial> {
    let (bytes, work) = graph
        .layout
        .equality_lookup_bound(entity, aspect, field)
        .ok_or_else(|| work_denial(field))?;
    admission
        .admit_read_scratch(bytes)
        .map_err(|stop| match stop {
            CompanionPreflightStop::PreparationMemoryExhausted { .. }
            | CompanionPreflightStop::PreparationMemoryCounterOverflow => WorthQueryApplicationQueryAdmissionDenial::new(
                WorthQueryApplicationQueryAdmissionDenialKind::ReadmissionPreparationMemoryExhausted,
                field,
            ),
            _ => work_denial(field),
        })?;
    admission
        .charge_external_work(work)
        .map_err(|_| work_denial(field))?;
    graph
        .layout
        .equality_field(entity, aspect, field)
        .and_then(|layout| layout.equality_index_id)
        .ok_or_else(|| denial(field))
}

fn work_denial(subject: &str) -> WorthQueryApplicationQueryAdmissionDenial {
    WorthQueryApplicationQueryAdmissionDenial::new(
        WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded,
        subject,
    )
}

fn denial(subject: &str) -> WorthQueryApplicationQueryAdmissionDenial {
    WorthQueryApplicationQueryAdmissionDenial::new(
        WorthQueryApplicationQueryAdmissionDenialKind::RuntimeSupportUnavailable,
        subject,
    )
}
