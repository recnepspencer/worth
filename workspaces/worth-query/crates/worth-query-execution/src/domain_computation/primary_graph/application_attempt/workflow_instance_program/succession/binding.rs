//! Bind the native completed-chain proof and existing output to one guarded candidate.
use super::super::super::{
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationRealizedEffect,
};
use super::{mismatch, WorkflowSuccessionAuthority, WorthQueryApplicationAttemptDenial};
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationCommitDenial,
};
pub(in crate::domain_computation::primary_graph::application_attempt) struct WorkflowSuccessionBindingProof
{
    binding: &'static str,
    role: &'static str,
    input_identity: [u8; 32],
    key_identity: [u8; 32],
    identity: [u8; 32],
    output_match: Option<worth_query_declaration::facade::application_schema::ApplicationOperationBodySuccessionOutputMatch>,
}
impl WorkflowSuccessionBindingProof {
    fn permits(
        &self,
        body: &worth_query_declaration::facade::application_schema::ApplicationOperationBodySuccession,
        idempotency: &WorthQueryApplicationIdempotencyBinding,
        handler: &super::super::super::read_set::MutationHandlerBindingProof,
    ) -> bool {
        self.binding == body.binding()
            && self.output_match.as_ref() == body.output_match()
            && self.role == body.role()
            && handler.binding == self.binding
            && handler.input_identity == self.input_identity
            && idempotency.key_identity() == &self.key_identity
            && idempotency.intent_identity() == &self.input_identity
            && idempotency.matches_workflow_instance(&self.identity)
    }
}
impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn bind_workflow_succession_authority(
        mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        mut authority: WorkflowSuccessionAuthority,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial> {
        let execution = phase
            .request_for_source(
                self.read_set
                    .lease
                    .handle()
                    .relational_runtime_instance_id(),
            )
            .map_err(|_| mismatch("foreign succession phase"))?;
        let admission = &self.read_set.admission;
        let body = admission
            .allowed_graph_contract()
            .body()
            .and_then(|body| body.succession())
            .ok_or_else(|| mismatch("body does not declare succession"))?;
        if admission.operation_scope_binding() != &authority.scope
            || admission.installed_capability_identity() != Some(authority.capability)
            || admission.governed_input_identity() != Some(&authority.input_identity)
            || self.read_set.lease.product().product_branch() != authority.branch
            || self.read_set.workflow_succession_binding.is_some()
            || self.read_set.workflow_birth_binding.is_some()
            || self.read_set.workflow_authority_binding.is_some()
            || body.binding() != authority.binding
            || body.role() != authority.role
            || body.output_match()
                != authority
                    .output_match
                    .as_ref()
                    .map(|matching| &matching.declaration)
        {
            return Err(mismatch("succession admission affinity changed"));
        }
        let subject = (authority.subject)(&self.output_correspondence)?;
        if subject != admission.scope_entity_id() {
            return Err(mismatch(
                "succession output is not this exact existing scope",
            ));
        }
        let output_match = authority.output_match.map(|matching| {
            let failure = match matching.target {
                Ok(target) if self.output_correspondence.matches_predecessor_output(&matching.declaration, target) => None,
                Ok(_) => Some(WorthQueryApplicationAttemptDenial::new(
                    super::WorthQueryApplicationAttemptDenialKind::WorkflowPredecessorOutputMismatch,
                    "selected output differs from the predecessor producer output",
                )),
                Err(denial) => Some(denial),
            };
            // Exact native replay resolves before this fresh-commit-only refusal.
            if authority.refusal.is_none() { authority.refusal = failure; }
            matching.declaration
        });
        self.read_set
            .append_authority_facts(&execution, &authority.facts, allocation)?;
        let layout = self.read_set.lease.layout.workflow();
        let mut effects = Vec::new();
        let start_revision = self.read_set.lease.handle().with_runtime(&execution, |runtime| {
            crate::domain_computation::primary_graph::workflow::instance::start_provenance::admit(
                &execution, runtime, self.read_set.lease.product().relational_basis(), authority.compiled.program_revision())
        })?;
        let successor = crate::domain_computation::primary_graph::workflow::instance::visit_instance_start_facts(layout, &authority.compiled,
            &authority.instance_identity, &start_revision, authority.branch.occurrence_ordinal(), subject, authority.carry,
            crate::domain_computation::primary_graph::workflow::actor::record::WorkflowAdmittedActor::from_admission(&self.read_set.admission),
            |effect| { effects.push(effect); Ok::<(), WorthQueryApplicationAttemptDenial>(()) })?;
        effects.push(WorthQueryApplicationRealizedEffect::CreateRelation {
            kind: layout.instance_succeeded_from_relation,
            key: "instance-succeeded-from".into(),
            from: worth_relational::facade::transactions::EntityReference::Created(successor),
            to: worth_relational::facade::transactions::EntityReference::Existing(
                authority.predecessor,
            ),
        });
        self.read_set
            .admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        self.effects.try_reserve(effects.len()).map_err(|_| {
            WorthQueryApplicationAttemptDenial::new(
                super::WorthQueryApplicationAttemptDenialKind::AllocationDenied,
                "succession effect storage allocation refused",
            )
        })?;
        self.effects.extend(effects);
        self.effect_posture = crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture::ApplicationWithPlatform;
        self.read_set.workflow_deadline = authority.carry.deadline;
        self.read_set.new_commit_refusal = authority.refusal;
        self.read_set.workflow_succession_binding = Some(WorkflowSuccessionBindingProof {
            binding: authority.binding,
            role: authority.role,
            input_identity: authority.input_identity,
            key_identity: authority.key_identity,
            identity: authority.identity,
            output_match,
        });
        Ok(self)
    }
}
pub(in crate::domain_computation::primary_graph::application_attempt) fn guard(
    succession: Option<
        &worth_query_declaration::facade::application_schema::ApplicationOperationBodySuccession,
    >,
    proof: Option<&WorkflowSuccessionBindingProof>,
    handler: Option<&super::super::super::read_set::MutationHandlerBindingProof>,
    idempotency: &WorthQueryApplicationIdempotencyBinding,
) -> Result<(), WorthQueryApplicationCommitDenial> {
    match (succession, proof, handler) {
        (None, None, _) => Ok(()),
        (Some(body), Some(proof), Some(handler)) if proof.permits(body, idempotency, handler) => {
            Ok(())
        }
        _ => Err(WorthQueryApplicationCommitDenial::workflow_authority_required()),
    }
}
