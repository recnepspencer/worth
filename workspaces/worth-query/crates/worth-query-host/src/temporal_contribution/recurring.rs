//! Stock recurring desktop rule: the canonical occurrence input, its
//! projection from a rule row, next-occurrence arithmetic and fresh owner
//! admission. The application binds these to its schema and owns the rule row.

use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryAdmittedAuthenticationAdapter,
    WorthQueryAuthenticationAdapterAdmissionDenial, WorthQueryCancellationSource,
    WorthQueryLocalProcessPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, StringApplicationValueBinding,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryInvariantMutationTarget, WorthQueryTemporalInvocationFailure,
    WorthQueryTemporalInvocationFailureKind, WorthQueryTemporalOccurrenceOutcome,
    WorthQueryTemporalPrincipalAdmission, WorthQueryTemporalPrincipalFailure,
    WorthQueryTemporalPrincipalFailureKind, WorthQueryTemporalPrincipalSource,
};
use worth_query_installation::facade::{
    WorthQueryClockCoordinate, WorthQueryConditionalNodeLocation, WorthQueryConditionalNodeRef,
    WorthQueryInstalledApplicationSchema, WorthQueryNamedClock, WorthQueryTemporalIntentBounds,
    WorthQueryTemporalIntentCandidate, WorthQueryTemporalIntentIdempotencyRelation,
    WorthQueryTemporalIntentIdentity, WorthQueryTemporalIntentProjectionFailure,
    WorthQueryTemporalIntentProjectionFailureKind, WorthQueryTemporalIntentProjector,
    WorthQueryTemporalOperationInputIdentity,
};

use worth_query_declaration::facade::application_schema::WorthQueryRecurringLifecycle;

/// One due occurrence of a recurring rule; the input of its fire operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRecurringOccurrence {
    pub rule_id: String,
    pub revision: u64,
    pub due_ns: u64,
    pub interval_ns: u64,
    pub max_fires: u64,
    pub fired_count: u64,
}
worth_query_declaration::worth_query_portable_type!(
    WorthQueryRecurringOccurrence => "worth.recurring.occurrence.v1"
);
worth_query_declaration::worth_query_structured_value_binding!(
    pub WorthQueryRecurringOccurrenceBinding for WorthQueryRecurringOccurrence {
        identity: "worth.recurring.occurrence.v1"
    }
);

/// A recurring rule's reconstruction row: the occurrence it schedules and its
/// lifecycle.
pub trait WorthQueryRecurringRow {
    fn occurrence(&self) -> WorthQueryRecurringOccurrence;
    fn lifecycle(&self) -> WorthQueryRecurringLifecycle;
}

/// Stable semantic identities of one recurring rule's installed parts.
pub trait WorthQueryRecurringRuleIdentity: 'static {
    const NODE: &'static str;
    const CLOCK: &'static str;
    const PROJECTOR: &'static str;
    const INVOKER: &'static str;
    const PRINCIPAL_SOURCE: &'static str;
}

/// Conditional node marker of recurring rule `Rule`.
pub struct WorthQueryRecurringNode<Rule>(PhantomData<fn() -> Rule>);

impl<Rule: WorthQueryRecurringRuleIdentity> WorthQueryRecurringNode<Rule> {
    /// The rule's operation conditional node, located by [`WorthQueryRecurringRuleIdentity::NODE`].
    ///
    /// # Panics
    ///
    /// Panics when `Rule::NODE` is not a valid conditional-node identity.
    pub fn reference() -> WorthQueryConditionalNodeRef<Rule, Rule, Rule, Self> {
        WorthQueryConditionalNodeRef::from_declared_location(
            WorthQueryConditionalNodeLocation::operation(Rule::NODE)
                .expect("a recurring rule declares a valid conditional-node identity"),
        )
    }
}

/// Caller-pumped named clock of recurring rule `Rule`.
pub struct WorthQueryRecurringClock<Rule>(PhantomData<fn() -> Rule>);

impl<Rule> std::fmt::Debug for WorthQueryRecurringClock<Rule> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("WorthQueryRecurringClock")
    }
}

impl<Rule: WorthQueryRecurringRuleIdentity> WorthQueryNamedClock
    for WorthQueryRecurringClock<Rule>
{
    const PORTABLE_IDENTITY: &'static str = Rule::CLOCK;
}

/// Projects a rule row into its timed intent: `Active` rows schedule their
/// due occurrence, `Completed` rows retire, and `Cancelled` or `Hub` rows cancel.
pub struct WorthQueryRecurringProjector<Rule>(PhantomData<fn() -> Rule>);

impl<Rule> Default for WorthQueryRecurringProjector<Rule> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<Rule, Node, Clock, Row>
    WorthQueryTemporalIntentProjector<Node, Clock, Row, WorthQueryRecurringOccurrence>
    for WorthQueryRecurringProjector<Rule>
where
    Rule: WorthQueryRecurringRuleIdentity,
    Row: WorthQueryRecurringRow,
{
    const SEMANTIC_IDENTITY: &'static str = Rule::PROJECTOR;

    fn project(
        &self,
        row: &Row,
    ) -> Result<
        WorthQueryTemporalIntentCandidate<Clock, WorthQueryRecurringOccurrence>,
        WorthQueryTemporalIntentProjectionFailure,
    > {
        let invalid = |detail: String| {
            WorthQueryTemporalIntentProjectionFailure::new(
                WorthQueryTemporalIntentProjectionFailureKind::InvalidIdentity,
                detail,
            )
        };
        let occurrence = row.occurrence();
        let identity = WorthQueryTemporalIntentIdentity::declare(occurrence.rule_id.clone())
            .map_err(|denial| invalid(denial.to_string()))?;
        let (input_identity, idempotency) = occurrence_identities(&occurrence).map_err(invalid)?;
        let due = WorthQueryClockCoordinate::from_nanoseconds(occurrence.due_ns);
        let candidate = match row.lifecycle() {
            WorthQueryRecurringLifecycle::Active => {
                WorthQueryTemporalIntentCandidate::active::<StringApplicationValueBinding>
            }
            WorthQueryRecurringLifecycle::Completed => {
                WorthQueryTemporalIntentCandidate::completed::<StringApplicationValueBinding>
            }
            WorthQueryRecurringLifecycle::Cancelled | WorthQueryRecurringLifecycle::Hub => {
                WorthQueryTemporalIntentCandidate::cancelled::<StringApplicationValueBinding>
            }
        };
        let (record, revision) = (occurrence.rule_id.clone(), occurrence.revision);
        candidate(
            identity,
            record,
            revision,
            due,
            occurrence,
            input_identity,
            idempotency,
        )
        .map_err(|_| invalid("recurring rule identity does not encode".to_owned()))
    }
}

/// Invoker identity for the application's recurring rule.
pub struct WorthQueryRecurringInvoker<Rule>(PhantomData<fn() -> Rule>);

impl<Rule> Default for WorthQueryRecurringInvoker<Rule> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// Decision of a recurring rule's schedule action: the hub scoping every
/// rule and, for a rule that already exists, its target and next revision.
pub struct WorthQueryRecurringScheduleDecision<Schema, Entity> {
    pub hub: WorthQueryInvariantMutationTarget<Schema, Entity>,
    pub existing: Option<(WorthQueryInvariantMutationTarget<Schema, Entity>, u64)>,
}

#[doc(hidden)]
pub fn recurring_invocation_failure(
    kind: WorthQueryTemporalInvocationFailureKind,
    cause: impl std::fmt::Debug,
) -> WorthQueryTemporalInvocationFailure {
    WorthQueryTemporalInvocationFailure::new(kind, format!("{cause:?}"))
}

/// Decides what follows the occurrence firing now: completion after
/// `max_fires` fires (or for a zero interval), otherwise the next occurrence
/// one interval later under `next_revision`.
pub fn recurring_next_occurrence(
    fired: &WorthQueryRecurringOccurrence,
    current_due_ns: u64,
    next_revision: u64,
) -> Result<
    WorthQueryTemporalOccurrenceOutcome<WorthQueryRecurringOccurrence>,
    WorthQueryTemporalInvocationFailure,
> {
    let fired_count = fired.fired_count.saturating_add(1);
    if fired_count >= fired.max_fires || fired.interval_ns == 0 {
        return Ok(WorthQueryTemporalOccurrenceOutcome::Complete);
    }
    let rejected = |detail: String| {
        WorthQueryTemporalInvocationFailure::new(
            WorthQueryTemporalInvocationFailureKind::InvocationRejected,
            detail,
        )
    };
    let due_ns = current_due_ns
        .checked_add(fired.interval_ns)
        .ok_or_else(|| rejected("recurring rule due time overflows".to_owned()))?;
    let next_input = WorthQueryRecurringOccurrence {
        revision: next_revision,
        due_ns,
        fired_count,
        ..fired.clone()
    };
    let (next_input_identity, next_idempotency) =
        occurrence_identities(&next_input).map_err(rejected)?;
    Ok(WorthQueryTemporalOccurrenceOutcome::Reschedule {
        next_due_nanoseconds: due_ns,
        next_input,
        next_input_identity,
        next_idempotency,
    })
}

fn occurrence_identities(
    occurrence: &WorthQueryRecurringOccurrence,
) -> Result<
    (
        WorthQueryTemporalOperationInputIdentity,
        WorthQueryTemporalIntentIdempotencyRelation,
    ),
    String,
> {
    let key = (&occurrence.rule_id, occurrence.revision, occurrence.due_ns);
    let input =
        WorthQueryTemporalOperationInputIdentity::declare(format!("{}:{}:{}", key.0, key.1, key.2))
            .map_err(|denial| denial.to_string())?;
    let idempotency = WorthQueryTemporalIntentIdempotencyRelation::declare(&key)
        .map_err(|denial| format!("{denial:?}"))?;
    Ok((input, idempotency))
}

/// Configuration of one installed recurring rule: the local owner whose fresh
/// authority reconstructs and fires occurrences, its admission clock and the
/// bounds of the rule's reconstructed intents.
pub struct WorthQueryRecurringConfig {
    owner: WorthQueryLocalProcessPrincipal,
    clock: WorthQueryAdmissionClock,
    bounds: Option<WorthQueryTemporalIntentBounds>,
}

/// Default reconstruction bounds: at most 64 reconstruction rows, 128 query
/// work units and 64 due wakes per observation.
const DEFAULT_RECURRING_BOUNDS: (usize, usize, usize) = (64, 128, 64);

impl WorthQueryRecurringConfig {
    pub fn new(owner: WorthQueryLocalProcessPrincipal, clock: WorthQueryAdmissionClock) -> Self {
        Self {
            owner,
            clock,
            bounds: None,
        }
    }

    /// Replaces the default reconstruction bounds.
    pub fn with_intent_bounds(mut self, bounds: WorthQueryTemporalIntentBounds) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// The configured or default reconstruction bounds.
    #[doc(hidden)]
    pub fn intent_bounds(&self) -> Result<WorthQueryTemporalIntentBounds, &'static str> {
        match self.bounds {
            Some(bounds) => Ok(bounds),
            None => {
                let (rows, work, wakes) = DEFAULT_RECURRING_BOUNDS;
                WorthQueryTemporalIntentBounds::new(rows, work, wakes)
            }
        }
    }

    /// Admits the configured owner against the installed schema.
    #[doc(hidden)]
    pub fn admit_owner<Schema: ApplicationSchema, Rule>(
        self,
        schema: &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<
        WorthQueryRecurringOwner<Schema, Rule>,
        WorthQueryAuthenticationAdapterAdmissionDenial,
    > {
        let adapter = self.owner.admit(schema, self.clock.clone())?;
        Ok(WorthQueryRecurringOwner {
            adapter: Arc::new(adapter),
            clock: self.clock,
            rule: PhantomData,
        })
    }
}

/// Fresh owner admission for each reconstruction and fire of rule `Rule`.
pub struct WorthQueryRecurringOwner<Schema, Rule> {
    adapter: Arc<WorthQueryAdmittedAuthenticationAdapter<Schema, WorthQueryLocalProcessPrincipal>>,
    clock: WorthQueryAdmissionClock,
    rule: PhantomData<fn() -> Rule>,
}

const OWNER_REQUEST_DEADLINE: Duration = Duration::from_secs(60);

impl<Schema, Rule> WorthQueryTemporalPrincipalSource<Schema>
    for WorthQueryRecurringOwner<Schema, Rule>
where
    Schema: ApplicationSchema + 'static,
    Rule: WorthQueryRecurringRuleIdentity,
{
    const SEMANTIC_IDENTITY: &'static str = Rule::PRINCIPAL_SOURCE;

    fn admit(
        &self,
    ) -> Result<WorthQueryTemporalPrincipalAdmission<Schema>, WorthQueryTemporalPrincipalFailure>
    {
        let scope = WorthQueryRequestScope::new(
            Some(self.clock.now() + OWNER_REQUEST_DEADLINE),
            WorthQueryCancellationSource::new().token(),
            self.clock.clone(),
        );
        let external = self.adapter.authenticate_local(&scope).map_err(|denial| {
            WorthQueryTemporalPrincipalFailure::new(
                WorthQueryTemporalPrincipalFailureKind::AdmissionRejected,
                format!("{denial:?}"),
            )
        })?;
        Ok(WorthQueryTemporalPrincipalAdmission::new(external, scope))
    }
}
