use crate::domain_computation::primary_graph::application_attempt::check_request_live;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

use super::effect_program::{admit_platform_effects, PlatformEffectDemand};
use super::{
    observe_adjacency, PublishedWorkflowDefinitionRef, WorthQueryApplicationAdjacencyDirection,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationObservedFact,
    WorthQueryCompleteApplicationReadSet, WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::primary_graph::workflow::{
    definition::{reconstruct_compiled_definition, WorkflowDefinitionCompilationPosture},
    instance::{visit_instance_start_facts, WorkflowLineageCarry},
};
use crate::domain_computation::runtime_time::WorthQueryRuntimeClock;

mod birth;
mod cancellation;
pub(in crate::domain_computation::primary_graph::application_attempt) use birth::{
    guard_birth, WorkflowBirthBindingProof,
};
pub use birth::{
    RequiredWorkflowBirth, WorkflowBirthAuthority, WorkflowBirthAuthoritySlot,
    WorthQueryWorkflowBirthAdapter,
};
mod discovery_intent;
pub use discovery_intent::WorthQueryWorkflowInstanceDiscoveryIntent;
mod intent_identity;
mod migration;
mod performed;
mod preparation;
mod publication;
mod start_basis;
mod supersession;

pub use cancellation::{
    PerformedWorkflowInstanceCancellation, PreparedWorkflowInstanceCancellation,
    WorkflowInstanceCancellationOutcome,
};
pub use preparation::{
    WorkflowInstanceBindingDenial, WorkflowInstancePreparationDenial,
    WorthQueryWorkflowInstanceAdapter,
};
pub use publication::{
    PerformedWorkflowInstanceStart, PreparedWorkflowInstanceStart, PublishedWorkflowInstanceRef,
    WorkflowInstanceStartOutcome,
};
pub use supersession::{RetiredWorkflowDefinitionStart, SupersededWorkflowDefinitionStart};

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
    pub(in crate::domain_computation::primary_graph) fn materialize_workflow_instance_start<
        Capability,
        Spec,
    >(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        published: PublishedWorkflowDefinitionRef,
        start_key_identity: [u8; 32],
        clock: &WorthQueryRuntimeClock,
    ) -> Result<
        PreparedWorkflowInstanceStart<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        self.authorize_instance_start::<Capability, Spec>(installed, published.branch())?;
        let (compiled, carry, supersession) =
            self.observe_workflow_start_basis(execution, installed, &published, clock)?;
        let layout = self.lease.layout.workflow().clone();
        let subject = self.admission.scope_entity_id();
        let discovery_intent =
            intent_identity::instance_identity(&compiled, subject, start_key_identity).map_err(
                |()| {
                    denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowInstanceIntentIdentityUnavailable,
                self.admission.operation(),
            )
                },
            )?;
        let start_revision = self.lease.handle().with_runtime(execution, |runtime| {
            crate::domain_computation::primary_graph::workflow::instance::start_provenance::admit(
                execution,
                runtime,
                self.lease.product().relational_basis(),
                compiled.program_revision(),
            )
        })?;
        let mut demand = PlatformEffectDemand::default();
        let branch_occurrence = self.lease.product().product_branch().occurrence_ordinal();
        visit_instance_start_facts(
            &layout,
            &compiled,
            discovery_intent.lookup_identity(),
            &start_revision,
            branch_occurrence,
            subject,
            carry,
            crate::domain_computation::primary_graph::workflow::actor::record::WorkflowAdmittedActor::from_admission(&self.admission),
            |effect| demand.observe(&effect),
        )?;
        let reservation = admit_platform_effects(&self, demand)?;
        let mut effects = Vec::new();
        visit_instance_start_facts(
            &layout,
            &compiled,
            discovery_intent.lookup_identity(),
            &start_revision,
            branch_occurrence,
            subject,
            carry,
            crate::domain_computation::primary_graph::workflow::actor::record::WorkflowAdmittedActor::from_admission(&self.admission),
            |effect| {
                effects.push(effect);
                Ok::<(), WorthQueryApplicationAttemptDenial>(())
            },
        )?;
        reservation.materialize(&effects)?;
        let start_path = compiled.start_path().to_owned();
        let definition = compiled.definition();
        let definition_content_identity = compiled.content_identity().clone();
        let program_revision = *compiled.program_revision();
        let program = WorthQueryApplicationEffectProgram {
            candidate_requirements: None,
            effect_backing_memory: None,
            read_set: self,
            effects,
            emission_retained_bytes: 0,
            emission_retained_bytes_ceiling: 0,
            conditional_definition: None,
            effect_posture: crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture::Platform,
            output_correspondence: Default::default(),
            retain_output_demand_observation: false,
            retain_client_observation: false,
            producer_required_invariants: &[],
            output_currentness_facts: None,
        };
        Ok(PreparedWorkflowInstanceStart {
            program,
            program_revision,
            definition,
            definition_content_identity,
            discovery_intent,
            instance_identity_locator: layout.instance.identity.clone(),
            start_path,
            supersession,
        })
    }
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
    /// Starting and migrating an instance share one authority: the workflow's
    /// installed start binding, on the branch the request selected.
    fn authorize_instance_start<Capability, Spec>(
        &self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        if branch != self.lease.product().product_branch() {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowInstanceAffinityMismatch,
                self.admission.operation(),
            ));
        }
        if !installed.instance_start_binding_matches::<Capability, Operation>()
            || self.admission.installed_capability_identity()
                != Some(*installed.instance_start_capability_identity_bytes())
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowInstanceAuthorityMismatch,
                self.admission.operation(),
            ));
        }
        Ok(())
    }
}

fn denial(
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(kind, subject)
}
