//! Index currency for the exact installed read that producer readmission will execute.

use std::sync::Arc;
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use worth_query_execution_lineage::facade::ports::PrimaryGraphInternalPort as _;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::indexes::DerivedIndexId;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

use super::super::WorthQueryApplicationQueryAdmissionDenial;
use super::super::WorthQueryApplicationQueryAdmissionDenialKind;
use crate::domain_computation::primary_graph::index_currency::{
    ensure_selected_field_index_slice_admitted, SelectedFieldIndexAdmissionStop,
};

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
    let (indexes, _allocation) = selected_indexes(execution, graph, query, scope_index)?;
    // The actual admitted basis pins currency; current security is still
    // selected and checked by the existing World/session owner on each read.
    let retained_basis = basis.clone();
    native.with_runtime(graph, execution, || SelectedFieldIndexAdmissionStop::Admission(CompanionPreflightStop::ForeignCell), |runtime| {
            ensure_selected_field_index_slice_admitted(execution, runtime, basis, &indexes)
        })
        .map_err(|stop| match stop {
            SelectedFieldIndexAdmissionStop::Admission(CompanionPreflightStop::PreparationMemoryCounterOverflow) => {
                WorthQueryApplicationQueryAdmissionDenial::new(
                    WorthQueryApplicationQueryAdmissionDenialKind::ReadmissionPreparationMemoryExhausted,
                    query.name(),
                )
            }
            SelectedFieldIndexAdmissionStop::Admission(CompanionPreflightStop::Execution(cause)) => {
                execution_denial(query.name(), cause)
            }
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
pub fn recover_recorded_query_indexes<Schema, Query, Parameters, QueryResult, Scope>(
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
    let (indexes, allocation) = selected_indexes(execution, graph, query, scope_index)?;
    allocation
        .claim_slice::<DerivedIndexId>(execution, indexes.len())
        .map_err(|cause| execution_denial(query.name(), cause))?;
    let mut missing = Vec::new();
    missing.try_reserve_exact(indexes.len()).map_err(|_| {
        WorthQueryApplicationQueryAdmissionDenial::new(
            WorthQueryApplicationQueryAdmissionDenialKind::ReadmissionPreparationMemoryExhausted,
            query.name(),
        )
    })?;
    super::super::ApplicationQueryNativeAccess::acquire().with_runtime(
        graph, execution, || denial(query.name()), |runtime| {
            for index in indexes {
                if !selected_index_is_current(execution, runtime, basis, index)
                    .map_err(|cause| execution_denial(query.name(), cause))?
                {
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
                DerivedIndexMaintenanceDenialKind::Execution(cause) => {
                    execution_denial(query.name(), cause)
                }
                _ => denial(query.name()),
            })?;
            Ok(())
        },
    )
}

fn selected_indexes<Schema, Query, Parameters, QueryResult, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &WorthQueryPrimaryGraph,
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    scope_index: DerivedIndexId,
) -> Result<(Vec<DerivedIndexId>, Arc<SourceFactsAllocation>), WorthQueryApplicationQueryAdmissionDenial>
{
    // Installed contract, root-selection mode, and optional live-target visits.
    let contract = query.read_family_binding().planning_contract();
    let predicate_count = if contract.root_paths().is_empty() {
        contract.predicates().len()
    } else {
        0
    };
    let count = predicate_count + usize::from(query.live().is_some()) + 1;
    let log = if count <= 1 {
        1_u64
    } else {
        u64::from(usize::BITS - (count - 1).leading_zeros()) + 1
    };
    let work_units = (count as u64)
        .checked_mul(log)
        .ok_or_else(|| execution_denial(query.name(), Cause::WorkCounterOverflow))?;
    SourceFactsAllocation::work(execution, work_units)
        .map_err(|cause| execution_denial(query.name(), cause))?;
    let allocation = SourceFactsAllocation::open(execution)
        .map_err(|cause| execution_denial(query.name(), cause))?;
    allocation
        .claim_slice::<DerivedIndexId>(execution, count)
        .map_err(|cause| execution_denial(query.name(), cause))?;
    let mut indexes = Vec::new();
    indexes.try_reserve_exact(count).map_err(|_| {
        WorthQueryApplicationQueryAdmissionDenial::new(
            WorthQueryApplicationQueryAdmissionDenialKind::ReadmissionPreparationMemoryExhausted,
            query.name(),
        )
    })?;
    if contract.root_paths().is_empty() {
        for predicate in contract.predicates() {
            indexes.push(selected_field_id(graph, predicate.field())?);
        }
    }
    if let Some(live) = query.live() {
        let field = live.target_identity();
        indexes.push(selected_field_id(
            graph,
            (field.entity(), field.aspect(), field.field()),
        )?);
    }
    indexes.push(scope_index);
    indexes.sort_unstable();
    indexes.dedup();
    Ok((indexes, allocation))
}

fn execution_denial(
    subject: &str,
    cause: Cause,
) -> WorthQueryApplicationQueryAdmissionDenial {
    let kind = match cause {
        Cause::PolicyMemoryExhausted { .. }
        | Cause::ProcessMemoryExhausted { .. }
        | Cause::DeclaredMemoryExhausted { .. }
        | Cause::MemoryLimitExceedsParent
        | Cause::ChargedBytesOverflow
        | Cause::MemoryOverflow => {
            WorthQueryApplicationQueryAdmissionDenialKind::ReadmissionPreparationMemoryExhausted
        }
        _ => WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded,
    };
    WorthQueryApplicationQueryAdmissionDenial::new(kind, subject)
}

fn selected_field_id(
    graph: &WorthQueryPrimaryGraph,
    (entity, aspect, field): (&str, &str, &str),
) -> Result<DerivedIndexId, WorthQueryApplicationQueryAdmissionDenial> {
    graph
        .layout_ref()
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
