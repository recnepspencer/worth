use super::super::super::{
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationRealizedEffect,
};
use super::{mismatch, WorkflowBirthAuthority, WorthQueryApplicationAttemptDenial};
use crate::domain_computation::primary_graph::application_attempt::effect_program::{
    admit_workflow_settlement_effects, PlatformEffectDemand, WorthQueryApplicationCreationPartition,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationCommitDenial,
};

pub(in crate::domain_computation::primary_graph::application_attempt) struct WorkflowBirthBindingProof
{
    binding: &'static str,
    role: &'static str,
    input_identity: [u8; 32],
    key_identity: [u8; 32],
    identity: [u8; 32],
}

impl WorkflowBirthBindingProof {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn permits(
        &self,
        body: &worth_query_declaration::facade::application_schema::ApplicationOperationBodyEnrollment,
        idempotency: &WorthQueryApplicationIdempotencyBinding,
        handler: &super::super::super::read_set::MutationHandlerBindingProof,
    ) -> bool {
        self.binding == body.binding()
            && self.role == body.role()
            && handler.binding == self.binding
            && handler.input_identity == self.input_identity
            && idempotency.key_identity() == &self.key_identity
            && idempotency.intent_identity() == &self.input_identity
            && idempotency.matches_workflow_birth(&self.identity)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn bind_workflow_birth_authority(
        mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        authority: WorkflowBirthAuthority,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial> {
        let execution = phase
            .request_for_source(
                self.read_set
                    .lease
                    .handle()
                    .relational_runtime_instance_id(),
            )
            .map_err(|_| mismatch("foreign birth phase"))?;
        let admission = &self.read_set.admission;
        let creation = admission
            .allowed_graph_contract()
            .body()
            .and_then(|body| body.creation())
            .ok_or_else(|| mismatch("birth requires installed creation body"))?;
        let enrollment = creation
            .enrollment()
            .ok_or_else(|| mismatch("body does not declare enrollment"))?;
        if admission.operation_scope_binding() != &authority.scope
            || admission.installed_capability_identity() != Some(authority.capability)
            || admission.governed_input_identity() != Some(&authority.input_identity)
            || self.read_set.lease.product().product_branch() != authority.branch
            || self.read_set.workflow_birth_binding.is_some()
            || self.read_set.workflow_authority_binding.is_some()
            || enrollment.binding() != authority.binding
            || enrollment.role() != authority.role
        {
            return Err(mismatch("birth admission affinity changed"));
        }
        let subject = (authority.subject)(&self.output_correspondence)?;
        let expected_kind = self
            .read_set
            .lease
            .layout
            .entity_kind(creation.entity())
            .ok_or_else(|| mismatch("birth subject kind is unavailable"))?;
        // The retained Create token must name this body's actual creation in
        // this candidate. Do not manufacture one from the role's text.
        if subject.kind_id != expected_kind || subject.client_key != worth_relational::facade::symbols::ClientKey::raw(creation.key())
            || self.effects.iter().filter(|effect| matches!(effect, WorthQueryApplicationRealizedEffect::CreateEntity { kind, key, partition: WorthQueryApplicationCreationPartition::Context(partition), .. }
                if *kind == subject.kind_id && key == creation.key() && *partition == subject.partition_id)).count() != 1 {
            return Err(mismatch("birth output does not belong to this creation candidate"));
        }
        self.read_set
            .append_authority_facts(&execution, &authority.facts, allocation)?;
        let mut effects = Vec::new();
        crate::domain_computation::primary_graph::workflow::instance::visit_instance_birth_facts(
            self.read_set.lease.layout.workflow(),
            &authority.compiled,
            &authority.instance_identity,
            authority.branch.occurrence_ordinal(),
            subject,
            authority.carry,
            |effect| {
                effects.push(effect);
                Ok::<(), WorthQueryApplicationAttemptDenial>(())
            },
        )?;
        let mut demand = PlatformEffectDemand::default();
        for effect in &effects {
            demand.observe(effect)?;
        }
        admit_workflow_settlement_effects(&self.read_set.admission, demand)?
            .materialize(&effects)?;
        self.effects.extend(effects);
        self.effect_posture = crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture::ApplicationWithPlatform;
        self.read_set.workflow_deadline = authority.carry.deadline;
        self.read_set.new_commit_refusal = authority.refusal;
        self.read_set.workflow_birth_binding = Some(WorkflowBirthBindingProof {
            binding: authority.binding,
            role: authority.role,
            key_identity: authority.key_identity,
            input_identity: authority.input_identity,
            identity: authority.identity,
        });
        Ok(self)
    }
}

pub(in crate::domain_computation::primary_graph::application_attempt) fn guard(
    enrollment: Option<
        &worth_query_declaration::facade::application_schema::ApplicationOperationBodyEnrollment,
    >,
    proof: Option<&WorkflowBirthBindingProof>,
    handler: Option<&super::super::super::read_set::MutationHandlerBindingProof>,
    idempotency: &WorthQueryApplicationIdempotencyBinding,
) -> Result<(), WorthQueryApplicationCommitDenial> {
    match (enrollment, proof, handler) {
        (None, None, _) => Ok(()),
        (Some(body), Some(proof), Some(handler)) if proof.permits(body, idempotency, handler) => {
            Ok(())
        }
        _ => Err(WorthQueryApplicationCommitDenial::workflow_authority_required()),
    }
}
