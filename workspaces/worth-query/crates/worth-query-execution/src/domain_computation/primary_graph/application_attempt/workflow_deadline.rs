//! A definition's total deadline bounds its instance's whole lineage on the
//! installed trusted clock. Preparation refuses a step once the deadline has
//! elapsed and the provider checks it again at commit; cancellation stays
//! open, so an overdue instance can always end.

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;
use worth_relational::facade::identity::{EntityId, KindId};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

use {super::observe_field_value, super::WorthQueryApplicationAttemptDenial, crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind, super::WorthQueryApplicationObservedFact, super::WorthQueryCompleteApplicationReadSet};
use crate::domain_computation::primary_graph::application_attempt::workflow::schema::WorthQueryWorkflowLayout;
use crate::domain_computation::runtime_time::WorthQueryRuntimeClock;

/// Deadlines are recorded as Unix-epoch milliseconds on the installed clock.
const TIMELINE: ApplicationCapabilityValidityTimeline =
    ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds;

/// The total deadline, in milliseconds, the definition declares.
pub(super) fn definition_deadline(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    definition: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Option<u64>, WorthQueryApplicationAttemptDenial> {
    positive_field(
        execution,
        (runtime, snapshot),
        (definition, layout.definition.entity_kind),
        &layout.definition.total_deadline_milliseconds,
        facts,
        "workflow definition total deadline is malformed",
    )
}

/// The Unix-epoch millisecond the instance's lineage must finish by.
pub(super) fn instance_deadline(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    instance: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Option<u64>, WorthQueryApplicationAttemptDenial> {
    positive_field(
        execution,
        (runtime, snapshot),
        (instance, layout.instance.entity_kind),
        &layout.instance.deadline,
        facts,
        "workflow instance deadline is malformed",
    )
}

/// The instant a new instance's lineage must finish by: the earlier of the
/// deadline it inherits and its own definition's, measured from now.
pub fn start_deadline(
    clock: &WorthQueryRuntimeClock,
    inherited: Option<u64>,
    declared: Option<u64>,
) -> Result<Option<u64>, WorthQueryApplicationAttemptDenial> {
    let Some(duration) = declared else {
        return Ok(inherited);
    };
    let own = now(clock)?.saturating_add(duration);
    Ok(Some(inherited.map_or(own, |inherited| inherited.min(own))))
}

/// Refuses once the deadline is no longer ahead on the installed clock. An
/// unreadable clock cannot show that it is.
pub fn ensure_before(
    clock: &WorthQueryRuntimeClock,
    deadline: u64,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    if now(clock)? < deadline {
        Ok(())
    } else {
        Err(WorthQueryApplicationAttemptDenial::new(
            WorthQueryApplicationAttemptDenialKind::WorkflowInstanceDeadlineElapsed,
            "workflow instance total deadline has elapsed",
        ))
    }
}

impl<Schema, Operation, Input, Scope, Phase>
    WorthQueryCompleteApplicationReadSet<Schema, Operation, Input, Scope, Phase>
{
    /// Reads the instance's deadline, refuses the step once it has elapsed,
    /// and keeps it so the provider checks it again at commit.
    pub(super) fn bind_workflow_deadline(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        clock: &WorthQueryRuntimeClock,
        layout: &WorthQueryWorkflowLayout,
        instance: EntityId,
    ) -> Result<Option<u64>, WorthQueryApplicationAttemptDenial> {
        let snapshot = self.lease.snapshot();
        let mut facts = Vec::with_capacity(1);
        let deadline = self.lease.handle().with_runtime(execution, |runtime| {
            instance_deadline(execution, runtime, snapshot, layout, instance, &mut facts)
        })?;
        self.append_completed_facts(
            facts,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )?;
        if let Some(deadline) = deadline {
            ensure_before(clock, deadline)?;
            self.workflow_deadline = Some(deadline);
        }
        Ok(deadline)
    }
}

fn positive_field(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    (runtime, snapshot): (&RelationalRuntime, &SnapshotHandle),
    (entity_id, kind): (EntityId, KindId),
    locator: &AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
    malformed: &'static str,
) -> Result<Option<u64>, WorthQueryApplicationAttemptDenial> {
    match observe_field_value(runtime, execution, snapshot, entity_id, kind, locator)? {
        None => {
            facts.push(WorthQueryApplicationObservedFact::AbsentField {
                entity_id,
                kind,
                locator: locator.clone(),
            });
            Ok(None)
        }
        Some(AspectValue::UInt64(value)) if value > 0 => {
            facts.push(WorthQueryApplicationObservedFact::Field {
                entity_id,
                kind,
                locator: locator.clone(),
                value: AspectValue::UInt64(value),
            });
            Ok(Some(value))
        }
        Some(_) => Err(WorthQueryApplicationAttemptDenial::new(
            WorthQueryApplicationAttemptDenialKind::InvalidAuthoritativeValue,
            malformed,
        )),
    }
}

pub fn now(clock: &WorthQueryRuntimeClock) -> Result<u64, WorthQueryApplicationAttemptDenial> {
    match clock.sample(TIMELINE).as_ref().map(|sample| sample.value()) {
        Ok(AspectValue::UInt64(now)) => Ok(*now),
        _ => Err(WorthQueryApplicationAttemptDenial::new(
            WorthQueryApplicationAttemptDenialKind::WorkflowTrustedTimeUnavailable,
            "the installed clock cannot bound a workflow deadline",
        )),
    }
}
