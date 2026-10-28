//! Index currency for the exact installed read that producer readmission will execute.

use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::indexes::DerivedIndexId;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::super::WorthQueryApplicationQueryAdmissionDenial;
use super::super::WorthQueryApplicationQueryAdmissionDenialKind;
use crate::domain_computation::primary_graph::index_currency::ensure_selected_field_indexes_admitted;
use crate::domain_computation::primary_graph::index_currency::SelectedFieldIndexAdmissionStop;

use crate::domain_computation::primary_graph::WorthQueryPrimaryGraph;

pub(in crate::domain_computation::primary_graph) use crate::domain_computation::primary_graph::selected_read_indexes::PreparedSelectedReadIndexes;

pub(in crate::domain_computation::primary_graph::application_query) fn ensure_query_truth_and_security_indexes_admitted<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Scope,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &WorthQueryPrimaryGraph,
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    truth: &AdmittedRelationalBranchBasis,
    security: &AdmittedRelationalBranchBasis,
) -> Result<PreparedSelectedReadIndexes, WorthQueryApplicationQueryAdmissionDenial> {
    ensure_query_truth_and_security_indexes_admitted_with_native_access(
        execution,
        graph,
        query,
        truth,
        security,
        &mut super::super::ApplicationQueryNativeAccess::acquire(),
    )
}

pub(in crate::domain_computation::primary_graph::application_query) fn ensure_query_truth_and_security_indexes_admitted_with_native_access<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Scope,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &WorthQueryPrimaryGraph,
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    truth: &AdmittedRelationalBranchBasis,
    security: &AdmittedRelationalBranchBasis,
    native: &mut super::super::ApplicationQueryNativeAccess<'_>,
) -> Result<PreparedSelectedReadIndexes, WorthQueryApplicationQueryAdmissionDenial> {
    let truth_indexes = prepare_selected_basis(execution, graph, query, truth, native)?;
    if truth.admission_identity() == security.admission_identity() {
        Ok(truth_indexes)
    } else {
        prepare_selected_basis(execution, graph, query, security, native)
    }
}

fn prepare_selected_basis<Schema, Query, Parameters, QueryResult, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &WorthQueryPrimaryGraph,
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    basis: &AdmittedRelationalBranchBasis,

    native: &mut super::super::ApplicationQueryNativeAccess<'_>,
) -> Result<PreparedSelectedReadIndexes, WorthQueryApplicationQueryAdmissionDenial> {
    // Installed contract, root-selection mode, and optional live-target visits.

    let contract = query.read_family_binding().planning_contract();
    let root = if contract.root_paths().is_empty() {
        match contract.predicates() {
            [] => None,
            [predicate] => Some(selected_field_id(graph, predicate.field())?),
            _ => return Err(denial(query.name())),
        }
    } else {
        None
    };
    let target = query
        .live()
        .map(|live| {
            let field = live.target_identity();
            selected_field_id(graph, (field.entity(), field.aspect(), field.field()))
        })
        .transpose()?;
    // The actual admitted basis pins currency; current security is still
    // selected and checked by the existing World/session owner on each read.

    let retained_basis = basis.clone();
    native.with_runtime_mut(graph, execution, || SelectedFieldIndexAdmissionStop::Admission(CompanionPreflightStop::ForeignCell), |runtime| {
            ensure_selected_field_indexes_admitted(execution, runtime, basis, [root, target])
        })
        .map_err(|stop| match stop {
            SelectedFieldIndexAdmissionStop::Admission( CompanionPreflightStop::PreparationMemoryCounterOverflow,
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
) -> Result<DerivedIndexId, WorthQueryApplicationQueryAdmissionDenial> {
    graph
        .layout
        .equality_field(entity, aspect, field)
        .and_then(|layout| layout.equality_index_id)
        .ok_or_else(|| denial(field))
}

fn denial(subject: &str) -> WorthQueryApplicationQueryAdmissionDenial {
    WorthQueryApplicationQueryAdmissionDenial::new(
        WorthQueryApplicationQueryAdmissionDenialKind::RuntimeSupportUnavailable,
        subject,
    )
}
