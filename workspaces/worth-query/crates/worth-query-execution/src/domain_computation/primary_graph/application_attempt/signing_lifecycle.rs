//! Native source-edit consequence progression, sharing one publication with its cause.
use super::snapshot_lease::WorthQueryApplicationSnapshotLease as Lease;
use super::{
    WorthQueryApplicationCommitDenial as Denial, WorthQueryApplicationCommitDenialKind as Kind,
    WorthQueryApplicationCommitOutcome as Outcome, WorthQueryApplicationObservedFact as Fact,
    WorthQueryApplicationRealizedEffect as Effect,
};
use crate::domain_computation::primary_graph::application_contribution::QueryRequestExecution;
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus as Locus;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledSigningLifecycleRule as Rule,
};
use worth_relational::facade::identity::{EntityId, KindId};
mod changes;
mod consequences;
mod guard;
mod observation;
mod signing;
pub(super) use guard::validate_incoming;
pub(super) use signing::capture_signing;

use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;

pub(super) struct PreparedLifecycle {
    pub(super) facts: FactPayload,
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
    facts: FactPayload,
    execution: &QueryRequestExecution<'_>,
    logical_request: [u8; 32],
) -> Result<PreparedLifecycle, Outcome> {
    let rules = admission.allowed_graph_contract().signing_lifecycle_rules();
    if rules.is_empty() {
        return Ok(PreparedLifecycle { facts });
    }
    let request = execution.execution_request().map_err(resource)?;
    let actor = admission.graph_work().principal();
    let batch = execution
        .decision_stage(|context| {
            let mut batch = consequences::Batch::default();
            for rule in rules {
                context.checkpoint(1).map_err(stopped)?;
                let changed = changes::select(rule, lease, effects, &facts, execution)?;
                for (subject, fields) in changed {
                    consequences::collect(
                        rule,
                        subject,
                        &fields,
                        actor,
                        admission.operation(),
                        logical_request,
                        lease,
                        &request,
                        context,
                        &mut batch,
                    )?;
                }
            }
            Ok::<_, Outcome>(batch)
        })
        .map_err(|cause| cause.into_commit_outcome())??;
    let extra = super::read_set::admit_completed_facts(
        batch.facts.len(),
        batch.facts.into_iter(),
        request.allocation_policy(),
        admission.publication_request(),
        admission.operation(),
        || {
            admission
                .validate_current_authority()
                .map_err(super::WorthQueryApplicationAttemptDenial::request_authority_lost)
        },
    )
    .map_err(attempt)?;
    let facts = facts
        .append_completed(
            extra,
            super::retained_decision_facts::StorageControl::new(
                request.allocation_policy(),
                Some(admission.publication_request()),
            ),
        )
        .map_err(|cause| attempt(cause.into_attempt_denial(admission.operation())))?;
    admission
        .validate_current_authority()
        .map_err(super::WorthQueryApplicationAttemptDenial::request_authority_lost)
        .map_err(attempt)?;
    effects
        .try_reserve(batch.effects.len())
        .map_err(|_| capacity())?;
    effects.extend(batch.effects);
    Ok(PreparedLifecycle { facts })
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
