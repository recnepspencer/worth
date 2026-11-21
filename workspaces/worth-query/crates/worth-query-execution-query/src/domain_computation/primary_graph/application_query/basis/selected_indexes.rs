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

pub use crate::domain_computation::primary_graph::selected_read_indexes::PreparedSelectedReadIndexes;

pub fn ensure_query_truth_and_security_indexes_admitted<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Scope,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &WorthQueryPrimaryGraph,
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    scope_index: DerivedIndexId,
    truth: &AdmittedRelationalBranchBasis,
    security: &AdmittedRelationalBranchBasis,
) -> Result<PreparedSelectedReadIndexes, WorthQueryApplicationQueryAdmissionDenial> {
    ensure_query_truth_and_security_indexes_admitted_with_native_access(
        execution,
        graph,
        query,
        scope_index,
        truth,
        security,
        &mut super::super::ApplicationQueryNativeAccess::acquire(),
    )
}

pub fn ensure_query_truth_and_security_indexes_admitted_with_native_access<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Scope,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &WorthQueryPrimaryGraph,
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    scope_index: DerivedIndexId,
    truth: &AdmittedRelationalBranchBasis,
    security: &AdmittedRelationalBranchBasis,
    native: &mut super::super::ApplicationQueryNativeAccess<'_>,
) -> Result<PreparedSelectedReadIndexes, WorthQueryApplicationQueryAdmissionDenial> {
    let truth_indexes =
        prepare_selected_basis(execution, graph, query, scope_index, truth, native)?;
    if truth.admission_identity() == security.admission_identity() {
        Ok(truth_indexes)
    } else {
        prepare_selected_basis(execution, graph, query, scope_index, security, native)
    }
}

fn prepare_selected_basis<Schema, Query, Parameters, QueryResult, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &WorthQueryPrimaryGraph,
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    scope_index: DerivedIndexId,
    basis: &AdmittedRelationalBranchBasis,

    native: &mut super::super::ApplicationQueryNativeAccess<'_>,
) -> Result<PreparedSelectedReadIndexes, WorthQueryApplicationQueryAdmissionDenial> {
    let indexes = selected_indexes(graph, query, scope_index)?;
    // The actual admitted basis pins currency; current security is still
    // selected and checked by the existing World/session owner on each read.
    let retained_basis = basis.clone();
    native.with_runtime(graph, execution, || SelectedFieldIndexAdmissionStop::Admission(CompanionPreflightStop::ForeignCell), |runtime| {
            ensure_selected_field_indexes_admitted(execution, runtime, basis, indexes)
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

/// Recorded original DATA may name a historical basis whose disposable indexes
/// were reclaimed. Only this explicit readmission door reconstructs those
/// installed indexes; fresh authority and scope admission precede the call.
pub fn recover_recorded_query_indexes<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Scope,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &WorthQueryPrimaryGraph,
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    scope_index: DerivedIndexId,
    basis: &AdmittedRelationalBranchBasis,
) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
    use crate::domain_computation::primary_graph::index_currency::selected_index_is_current;
    use worth_relational::facade::indexes::{
        DerivedIndexBuildRequest, DerivedIndexMaintenanceDenialKind,
    };
    let indexes = selected_indexes(graph, query, scope_index)?;
    super::super::ApplicationQueryNativeAccess::acquire().with_runtime(
        graph, execution, || denial(query.name()), |runtime| {
            let mut missing = Vec::new();
            for index in indexes.into_iter().flatten() {
                if !missing.contains(&index) && !selected_index_is_current(execution, runtime, basis, index)
                    .map_err(|_| WorthQueryApplicationQueryAdmissionDenial::new(
                        WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded, query.name()))? {
                    missing.push(index);
                }
            }
            if missing.is_empty() { return Ok(()); }
            let observation = basis.observation();
            let source_commit_id = observation.commit_id().ok_or_else(|| denial(query.name()))?;
            runtime.index_authority().reconstruct_for_basis(
                execution,
                DerivedIndexBuildRequest {
                    source_commit_id,
                    branch_id: observation.identity().branch_id().clone(),
                    index_ids: missing,
                },
                basis,
                crate::domain_computation::primary_graph::index_maintenance_budget::cold_index_reconstruction_budget(),
            ).map_err(|stop| match stop.kind {
                DerivedIndexMaintenanceDenialKind::Execution(_) => WorthQueryApplicationQueryAdmissionDenial::new(
                    WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded, query.name()),
                _ => denial(query.name()),
            })?;
            Ok(())
        },
    )
}

fn selected_indexes<Schema, Query, Parameters, QueryResult, Scope>(
    graph: &WorthQueryPrimaryGraph,
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    scope_index: DerivedIndexId,
) -> Result<[Option<DerivedIndexId>; 3], WorthQueryApplicationQueryAdmissionDenial> {
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
    Ok([root, target, Some(scope_index)])
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
