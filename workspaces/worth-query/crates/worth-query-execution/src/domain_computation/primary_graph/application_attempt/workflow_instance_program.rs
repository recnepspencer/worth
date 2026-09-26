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
    instance::visit_instance_start_facts,
};

mod cancellation;
mod intent_identity;
mod migration;
mod performed;
mod preparation;
mod publication;

pub use cancellation::{
    PerformedWorkflowInstanceCancellation, PreparedWorkflowInstanceCancellation,
    WorkflowInstanceCancellationOutcome,
};
pub use preparation::{
    WorkflowInstanceBindingDenial, WorkflowInstancePreparationDenial,
    WorthQueryWorkflowInstanceStartAdapter,
};
pub use publication::{
    PerformedWorkflowInstanceStart, PreparedWorkflowInstanceStart, PublishedWorkflowInstanceRef,
    WorkflowInstanceStartOutcome,
};

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
        Program,
    >(
        mut self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        published: PublishedWorkflowDefinitionRef,
        start_key_identity: [u8; 32],
    ) -> Result<
        PreparedWorkflowInstanceStart<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        self.authorize_instance_start::<Capability, Spec, Program>(installed, published.branch())?;
        let layout = self.lease.layout.workflow().clone();
        let (compiled, mut compile_facts) = reconstruct_compiled_definition(
            self.lease.handle(),
            self.lease.snapshot(),
            &layout,
            &published,
            installed.program_revision(),
            Spec::IDENTITY.as_str(),
            installed.support_identity_bytes(),
            usize::from(installed.resources().maximum_definition_nodes()),
            usize::from(installed.resources().maximum_definition_connections()),
            WorkflowDefinitionCompilationPosture::Current,
        )?;
        let maximum_instances = usize::try_from(installed.resources().maximum_live_instances())
            .map_err(|_| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
                    self.admission.operation(),
                )
            })?;
        let instances = self
            .lease
            .handle()
            .with_runtime(|runtime| {
                observe_adjacency(
                    runtime,
                    self.lease.snapshot(),
                    layout.live_instance_lineage_relation,
                    compiled.lineage(),
                    WorthQueryApplicationAdjacencyDirection::Incoming,
                    maximum_instances.saturating_mul(2).saturating_add(1),
                )
            })
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
                    self.admission.operation(),
                )
            })?;
        compile_facts.push(
            WorthQueryApplicationObservedFact::WorkflowInstanceCapacity {
                relation_kind: layout.live_instance_lineage_relation,
                lineage: compiled.lineage(),
                maximum_instances,
                instances,
            },
        );
        if self.facts.len().saturating_add(compile_facts.len())
            > self
                .admission
                .allowed_graph_contract()
                .decision_fact_budget()
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
                self.admission.operation(),
            ));
        }
        self.facts.extend(compile_facts);
        let subject = self.admission.scope_entity_id();
        let (instance_identity, instance_intent_identity) =
            intent_identity::instance_identity(&compiled, subject, start_key_identity).map_err(
                |()| {
                    denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowInstanceIntentIdentityUnavailable,
                self.admission.operation(),
            )
                },
            )?;
        let mut demand = PlatformEffectDemand::default();
        let branch_occurrence = self.lease.product().product_branch().occurrence_ordinal();
        visit_instance_start_facts(
            &layout,
            &compiled,
            &instance_identity,
            branch_occurrence,
            subject,
            |effect| demand.observe(&effect),
        )?;
        let reservation = admit_platform_effects(&self, demand)?;
        let mut effects = Vec::new();
        visit_instance_start_facts(
            &layout,
            &compiled,
            &instance_identity,
            branch_occurrence,
            subject,
            |effect| {
                effects.push(effect);
                Ok::<(), WorthQueryApplicationAttemptDenial>(())
            },
        )?;
        let validator_work_admission = reservation.materialize(&effects)?;
        let start_path = compiled.start_path().to_owned();
        let definition = compiled.definition();
        let definition_content_identity = compiled.content_identity().clone();
        let program_revision = compiled.program_revision().clone();
        let program = WorthQueryApplicationEffectProgram {
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
        };
        Ok(PreparedWorkflowInstanceStart {
            program,
            program_revision,
            definition,
            definition_content_identity,
            instance_identity,
            instance_intent_identity,
            instance_identity_locator: layout.instance.identity.clone(),
            start_path,
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
    fn authorize_instance_start<Capability, Spec, Program>(
        &self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
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
