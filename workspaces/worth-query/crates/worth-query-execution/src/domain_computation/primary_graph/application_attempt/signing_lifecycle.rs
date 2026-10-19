//! Native source-edit consequence progression, sharing one publication with its cause.
use super::snapshot_lease::WorthQueryApplicationSnapshotLease as Lease;
use super::{
    WorthQueryApplicationCommitDenial as Denial, WorthQueryApplicationCommitDenialKind as Kind,
    WorthQueryApplicationCommitOutcome as Outcome, WorthQueryApplicationObservedFact as Fact,
    WorthQueryApplicationRealizedEffect as Effect,
};
use crate::domain_computation::primary_graph::application_contribution::{
    QueryMemoryReservation, QueryRequestExecution,
};
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus as Locus;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledSigningLifecycleRule as Rule,
};
use worth_relational::facade::identity::{EntityId, KindId};
mod changes;
mod consequences;
mod graph;
mod guard;
mod observation;
mod signing;
pub(super) use guard::validate_incoming;
pub(super) use signing::capture_signing;

pub(super) struct PreparedLifecycle {
    pub(super) facts: worth_execution::ExecutionArray<Fact>,
    pub(super) memory: Option<LifecycleMemory>,
}
pub(super) struct LifecycleMemory {
    _base: QueryMemoryReservation,
    _graph: Vec<QueryMemoryReservation>,
}

pub(super) fn expand<Schema: ApplicationSchema, Operation, Input, Scope>(
    admission: &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        Schema,
        Operation,
        Input,
        Scope,
    >,
    lease: &Lease,
    effects: &mut Vec<Effect>,
    facts: worth_execution::ExecutionArray<Fact>,
    execution: &QueryRequestExecution<'_>,
    logical_request: [u8; 32],
) -> Result<PreparedLifecycle, Outcome> {
    let rules = admission.allowed_graph_contract().signing_lifecycle_rules();
    if rules.is_empty() {
        return Ok(PreparedLifecycle {
            facts,
            memory: None,
        });
    }
    let ceiling = admission
        .allowed_graph_contract()
        .signing_lifecycle_ceiling()
        .ok_or_else(unavailable)?;
    let retained = ceiling
        .resources()
        .maximum_retained_representation_bytes()
        .checked_mul(3)
        .ok_or_else(capacity)?;
    let memory = execution
        .reserve(u64::try_from(retained).map_err(|_| capacity())?)
        .map_err(resource)?;
    let request = execution.execution_request().map_err(resource)?;
    let actor = admission.graph_work().principal();
    let batch = execution
        .decision_stage(|context| {
            let mut batch = consequences::Batch::default();
            for rule in rules {
                context.checkpoint(1).map_err(stopped)?;
                let changed = changes::select(rule, lease, effects, &facts, execution)?;
                let subjects = graph::affected_subjects(
                    rule, lease, effects, &changed, execution, &request, context, &mut batch,
                )?;
                for subject in subjects {
                    consequences::collect(
                        rule,
                        subject,
                        &changed,
                        effects,
                        actor,
                        admission.operation(),
                        logical_request,
                        lease,
                        execution,
                        &request,
                        context,
                        &mut batch,
                    )?;
                }
            }
            batch.admit(admission)?;
            Ok::<_, Outcome>(batch)
        })
        .map_err(|cause| cause.into_commit_outcome())??;
    let total = facts
        .len()
        .checked_add(batch.facts.len())
        .ok_or_else(capacity)?;
    let facts = super::read_set::admit_array(
        total,
        facts.into_iter().chain(batch.facts),
        request.allocation_policy(),
        admission.operation(),
        || {
            admission
                .validate_current_authority()
                .map_err(super::WorthQueryApplicationAttemptDenial::request_authority_lost)
        },
    )
    .map_err(attempt)?;
    effects.extend(batch.effects);
    Ok(PreparedLifecycle {
        facts,
        memory: Some(LifecycleMemory {
            _base: memory,
            _graph: batch.memory,
        }),
    })
}

fn refused(kind: Kind) -> Outcome {
    Outcome::Denied(Denial::signing_lifecycle(kind))
}
fn unavailable() -> Outcome {
    refused(Kind::SigningLifecycleUnavailable)
}
fn capacity() -> Outcome {
    refused(Kind::SigningLifecycleRepresentationCapacity)
}
fn attempt(cause: super::WorthQueryApplicationAttemptDenial) -> Outcome {
    match cause.kind() {
        super::WorthQueryApplicationAttemptDenialKind::Cancelled => Outcome::Cancelled,
        super::WorthQueryApplicationAttemptDenialKind::DeadlineExceeded => Outcome::TimedOut,
        _ => Outcome::Denied(Denial::signing_lifecycle_attempt(cause)),
    }
}
fn resource(
    cause: crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial,
) -> Outcome {
    crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(cause)
        .into_commit_outcome()
}
fn stopped(cause: worth_execution::MapKernelStop) -> Outcome {
    crate::domain_computation::primary_graph::application_contribution::advancement_denial(
        worth_execution::WorkCeilingDenial::Stopped(cause),
    )
    .into_commit_outcome()
}
fn coordinates(
    entity: EntityId,
) -> worth_query_declaration::facade::application_schema::ApplicationSigningEntityCoordinates {
    worth_query_declaration::facade::application_schema::ApplicationSigningEntityCoordinates {
        partition: entity.partition_value_u64(),
        slot: entity.local_slot_value(),
        generation: entity.generation_value(),
    }
}
