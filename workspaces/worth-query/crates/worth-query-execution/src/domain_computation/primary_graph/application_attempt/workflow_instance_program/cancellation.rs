//! An explicit cancellation ends a live instance where it stands. It is not
//! rollback: every effect the instance performed remains, and the outcome
//! reports each one. A cancellation prepared before a step settles goes stale,
//! and a step admitted before the cancellation commits goes stale in turn.
use crate::domain_computation::primary_graph::application_attempt::check_request_live;

mod close;
mod publication;
mod recorded;

use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

use super::super::effect_program::{admit_platform_effects, PlatformEffectDemand};
use super::super::workflow_instance_observation::instance_binding::deny_ended;
use super::super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEffectProgram, WorthQueryCompleteApplicationReadSet,
    WorthQueryProjectedApplicationMutation,
};
use super::{intent_identity, PublishedWorkflowInstanceRef};
use close::ClosedWorkflowInstance;
pub use publication::{
    PerformedWorkflowInstanceCancellation, PreparedWorkflowInstanceCancellation,
    WorkflowInstanceCancellationOutcome,
};

const HISTORY: WorthQueryApplicationAttemptDenialKind =
    WorthQueryApplicationAttemptDenialKind::WorkflowInstanceHistoryUnavailable;

fn maximum_transitions<Schema, Spec>(
    installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
) -> usize
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    usize::try_from(
        installed
            .resources()
            .maximum_retained_transitions_per_instance(),
    )
    .unwrap_or(usize::MAX)
}

impl<Schema, Operation, Input, Scope>
    WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
{
    pub(in crate::domain_computation::primary_graph) fn materialize_workflow_instance_cancellation<
        Capability,
        Spec,
    >(
        mut self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        instance: PublishedWorkflowInstanceRef,
        cancel_key_identity: [u8; 32],
    ) -> Result<
        PreparedWorkflowInstanceCancellation<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let branch = self.lease.product().product_branch();
        // The start capability authorizes ending what it started, on the
        // branch whose copy the request ends.
        self.authorize_instance_start::<Capability, Spec>(installed, branch)?;
        if instance.branch() != branch {
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowInstanceAffinityMismatch,
                self.admission.operation(),
            ));
        }
        let (identity, intent_identity) = intent_identity::cancellation_identity(
            instance.entity_id(),
            branch.occurrence_ordinal(),
            cancel_key_identity,
        )
        .map_err(|()| {
            WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowInstanceIntentIdentityUnavailable,
                self.admission.operation(),
            )
        })?;
        let layout = self.lease.layout.workflow().clone();
        let snapshot = self.lease.snapshot();
        // The exact replay resolves before the definition is read; any other
        // request for an ended instance is refused by name.
        let recorded = self.lease.handle().with_runtime(|runtime| {
            let recorded = recorded::observe_recorded_cancellation(
                runtime,
                snapshot,
                &layout,
                instance.entity_id(),
                &identity,
                maximum_transitions(installed),
                installed.resources().history_reconstruction_budget(),
            )?;
            if recorded.is_none() {
                deny_ended(runtime, snapshot, &layout, instance.entity_id())?;
            }
            Ok::<_, WorthQueryApplicationAttemptDenial>(recorded)
        })?;
        let ClosedWorkflowInstance {
            effects,
            performed,
            facts,
        } = match recorded {
            Some((performed, facts)) => ClosedWorkflowInstance {
                effects: Vec::new(),
                performed,
                facts,
            },
            None => self.close_live_workflow_instance(installed, &layout, &instance, &identity)?,
        };
        check_request_live(
            self.admission.publication_request(),
            self.admission.operation(),
        )?;
        self.append_completed_facts(
            facts,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )?;
        let mut demand = PlatformEffectDemand::default();
        for effect in &effects {
            demand.observe(effect)?;
        }
        let reservation = admit_platform_effects(&self, demand)?;
        let validator_work_admission = reservation.materialize(&effects)?;
        Ok(PreparedWorkflowInstanceCancellation {
            program_revision: *installed.program_revision(),
            instance,
            cancellation_identity: identity,
            intent_identity,
            cancellation_identity_locator: layout.instance.cancellation_identity.clone(),
            performed: performed.into_iter().map(|effect| effect.path).collect(),
            program: WorthQueryApplicationEffectProgram {
                read_set: self,
                effects,
                emission_retained_bytes: 0,
                emission_retained_bytes_ceiling: 0,
                conditional_definition: None,
                effect_posture: crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture::Platform,
                validator_work_admission,
                output_correspondence: Default::default(),
                retain_output_demand_observation: false,
                retain_client_observation: false,
                producer_required_invariants: &[],
                output_currentness_facts: None,
            },
        })
    }
}
