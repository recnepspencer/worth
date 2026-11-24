//! Caller-owned projector, recurring invoker, and fire log for desktop temporal catch-up.

mod mutation;
mod principal;
pub use mutation::*;
pub use principal::*;

use std::sync::{Arc, Mutex};
use worth_query_decl::facade::application_schema::StringApplicationValueBinding;
use worth_query_host::facade::{
    domain,
    primary_graph::{self, WorthQueryInvariantMutationTarget, WorthQueryTemporalOccurrenceOutcome},
    temporal_contribution::WorthQueryDesktopTemporalNodeSpec,
    worth_query_conditional_node,
};

use super::schema::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopReminderDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopReminderOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopReminderFamily;

worth_query_conditional_node!(
    pub DesktopReminderNode in DesktopReminderDomain, DesktopReminderOperation, DesktopReminderFamily
    => operation "desktop-reminder-due"
);

#[derive(Debug)]
pub struct DesktopReminderClock;
impl domain::WorthQueryNamedClock for DesktopReminderClock {
    const PORTABLE_IDENTITY: &'static str = "worth.query.court.desktop-reminder.clock.v1";
}

pub fn desktop_node_spec() -> WorthQueryDesktopTemporalNodeSpec {
    WorthQueryDesktopTemporalNodeSpec::from_fields(
        "desktop_reminder_due_gate",
        1,
        "desktop-reminder-due",
        identity_field(),
        gate_field(),
        "desktop.reminder.temporal",
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FiredOccurrence {
    pub id: String,
    pub rev: u64,
    pub due: u64,
    pub fired_count_after: u64,
}

#[derive(Clone, Default)]
pub struct ReminderFireLog {
    entries: Arc<Mutex<Vec<FiredOccurrence>>>,
}
impl ReminderFireLog {
    pub fn snapshot(&self) -> Vec<FiredOccurrence> {
        self.entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
    fn record(&self, entry: FiredOccurrence) {
        self.entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(entry);
    }
}

pub struct ReminderProjector;
impl
    domain::WorthQueryTemporalIntentProjector<
        DesktopReminderNode,
        DesktopReminderClock,
        ReminderRow,
        FireReminderInput,
    > for ReminderProjector
{
    const SEMANTIC_IDENTITY: &'static str = "worth.query.court.desktop-reminder.projector.v1";

    fn project(
        &self,
        row: &ReminderRow,
    ) -> Result<
        domain::WorthQueryTemporalIntentCandidate<DesktopReminderClock, FireReminderInput>,
        domain::WorthQueryTemporalIntentProjectionFailure,
    > {
        let identity = domain::WorthQueryTemporalIntentIdentity::declare(row.identity.clone())
            .map_err(projection_failure)?;
        let input_identity = domain::WorthQueryTemporalOperationInputIdentity::declare(format!(
            "{}:{}:{}",
            row.identity, row.revision, row.due
        ))
        .map_err(projection_failure)?;
        let idempotency = domain::WorthQueryTemporalIntentIdempotencyRelation::declare(&(
            &row.identity,
            row.revision,
            row.due,
        ))
        .map_err(|denial| projection_failure(format!("{denial:?}")))?;
        let due = domain::WorthQueryClockCoordinate::from_nanoseconds(row.due);
        let input = FireReminderInput {
            identity: row.identity.clone(),
            revision: row.revision,
            due: row.due,
            interval: row.interval,
            max_fires: row.max_fires,
            fired_count: row.fired_count,
        };
        match row.lifecycle.as_str() {
            "active" => domain::WorthQueryTemporalIntentCandidate::active::<
                StringApplicationValueBinding,
            >(
                identity,
                row.identity.clone(),
                row.revision,
                due,
                input,
                input_identity,
                idempotency,
            ),
            "completed" => domain::WorthQueryTemporalIntentCandidate::completed::<
                StringApplicationValueBinding,
            >(
                identity,
                row.identity.clone(),
                row.revision,
                due,
                input,
                input_identity,
                idempotency,
            ),
            _ => domain::WorthQueryTemporalIntentCandidate::cancelled::<
                StringApplicationValueBinding,
            >(
                identity,
                row.identity.clone(),
                row.revision,
                due,
                input,
                input_identity,
                idempotency,
            ),
        }
        .map_err(|_| projection_failure("reminder record identity must encode"))
    }
}

fn projection_failure(
    detail: impl Into<String>,
) -> domain::WorthQueryTemporalIntentProjectionFailure {
    domain::WorthQueryTemporalIntentProjectionFailure::new(
        domain::WorthQueryTemporalIntentProjectionFailureKind::InvalidIdentity,
        detail,
    )
}

pub struct ReminderInvoker {
    pub fire_log: ReminderFireLog,
}

impl
    primary_graph::WorthQueryTemporalOperationInvoker<
        DesktopReminderSchema,
        FireReminder,
        FireReminderInput,
        ReminderIntent,
    > for ReminderInvoker
{
    const SEMANTIC_IDENTITY: &'static str = "worth.query.court.desktop-reminder.invoker.v1";
    type Projection = WorthQueryInvariantMutationTarget<DesktopReminderSchema, ReminderIntent>;

    fn preconditions(
        &self,
        _input: &FireReminderInput,
    ) -> worth_query_decl::facade::application_schema::TypedMutationPreconditions<
        DesktopReminderSchema,
        FireReminder,
        ReminderIntent,
    > {
        Default::default()
    }

    fn project(
        &self,
        input: &FireReminderInput,
        reader: &mut primary_graph::WorthQueryApplicationOperationInvariantProjectionReader<
            '_,
            '_,
            DesktopReminderSchema,
            FireReminder,
        >,
        _scope: &primary_graph::WorthQueryInvariantEntityIdentity<
            DesktopReminderSchema,
            ReminderIntent,
        >,
    ) -> Result<Self::Projection, primary_graph::WorthQueryTemporalInvocationFailure> {
        let intent = reader
            .resolve_entity(identity_field(), input.identity.clone())
            .map_err(projection_rejected)?;
        reader
            .require_decision_field(&intent, identity_field())
            .map_err(projection_rejected)?;
        reader
            .decision_field(&intent, fired_count_field())
            .map_err(projection_rejected)?;
        reader.mutation_target(&intent).map_err(projection_rejected)
    }

    fn next_occurrence(
        &self,
        input: &FireReminderInput,
        _projection: &Self::Projection,
        current_due_nanoseconds: u64,
        next_revision: u64,
    ) -> Result<
        WorthQueryTemporalOccurrenceOutcome<FireReminderInput>,
        primary_graph::WorthQueryTemporalInvocationFailure,
    > {
        let next_count = input.fired_count.saturating_add(1);
        if next_count >= input.max_fires || input.interval == 0 {
            return Ok(WorthQueryTemporalOccurrenceOutcome::Complete);
        }
        let next_due = current_due_nanoseconds
            .checked_add(input.interval)
            .ok_or_else(|| invocation_rejected("recurring next due overflow"))?;
        let next_input = FireReminderInput {
            identity: input.identity.clone(),
            revision: next_revision,
            due: next_due,
            interval: input.interval,
            max_fires: input.max_fires,
            fired_count: next_count,
        };
        let next_input_identity = domain::WorthQueryTemporalOperationInputIdentity::declare(
            format!("{}:{}:{}", input.identity, next_revision, next_due),
        )
        .map_err(invocation_rejected)?;
        let next_idempotency = domain::WorthQueryTemporalIntentIdempotencyRelation::declare(&(
            &input.identity,
            next_revision,
            next_due,
        ))
        .map_err(|denial| invocation_rejected(format!("{denial:?}")))?;
        Ok(WorthQueryTemporalOccurrenceOutcome::Reschedule {
            next_due_nanoseconds: next_due,
            next_input,
            next_input_identity,
            next_idempotency,
        })
    }

    fn apply(
        &self,
        input: FireReminderInput,
        target: Self::Projection,
        effects: &mut primary_graph::WorthQueryApplicationEffectProgramBuilder<
            DesktopReminderSchema,
            FireReminder,
            FireReminderInput,
            ReminderIntent,
        >,
    ) -> Result<(), primary_graph::WorthQueryTemporalInvocationFailure> {
        let next_count = input.fired_count.saturating_add(1);
        let entity = effects
            .projected_entity(&target)
            .map_err(invocation_rejected)?;
        effects
            .write_field(&entity, fired_count_field(), next_count)
            .map_err(invocation_rejected)?;
        self.fire_log.record(FiredOccurrence {
            id: input.identity,
            rev: input.revision,
            due: input.due,
            fired_count_after: next_count,
        });
        Ok(())
    }
}

fn projection_rejected(error: impl ToString) -> primary_graph::WorthQueryTemporalInvocationFailure {
    primary_graph::WorthQueryTemporalInvocationFailure::new(
        primary_graph::WorthQueryTemporalInvocationFailureKind::ProjectionRejected,
        error.to_string(),
    )
}
fn invocation_rejected(error: impl ToString) -> primary_graph::WorthQueryTemporalInvocationFailure {
    primary_graph::WorthQueryTemporalInvocationFailure::new(
        primary_graph::WorthQueryTemporalInvocationFailureKind::InvocationRejected,
        error.to_string(),
    )
}
