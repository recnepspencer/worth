//! Receipt projection describes the one native birth; it cannot authorize it.
use super::super::super::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationIdempotencyBinding,
};
use super::super::PublishedWorkflowInstanceRef;
use super::{
    mismatch, ApplicationMutationBinding, ApplicationSchema, RequiredWorkflowBirth, TypeId,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationOutputRole,
    WorthQueryWorkflowBirthAdapter,
};

impl RequiredWorkflowBirth {
    pub fn instance_in(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<PublishedWorkflowInstanceRef, WorthQueryApplicationAttemptDenial> {
        let idempotency = receipt.idempotency_binding();
        if receipt.product_branch() != self.branch
            || receipt.principal_scope() != &self.scope
            || idempotency.key_identity() != &self.key_identity
            || idempotency.intent_identity() != &self.input_identity
            || !idempotency.matches_workflow_birth(&self.identity)
            || !idempotency.is_for_mutation_binding(self.binding)
        {
            return Err(mismatch("receipt does not describe this birth request"));
        }
        (self.subject_in_receipt)(receipt)?;
        super::super::publication::project_instance(
            receipt,
            self.compiled.content_identity(),
            self.compiled.definition(),
            *self.compiled.program_revision(),
            &self.instance_identity,
            &self.identity_locator,
            self.compiled.start_path(),
        )
        .ok_or_else(|| mismatch("receipt does not record exactly one native birth instance"))
    }
}

pub(super) fn subject_in_receipt<Schema, Binding, Role>(
    receipt: &WorthQueryApplicationCommitReceipt,
) -> Result<worth_relational::facade::identity::EntityId, WorthQueryApplicationAttemptDenial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Role: WorthQueryApplicationOutputRole<Schema = Schema, Contract = Binding::Output>,
{
    use crate::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::OutputRoleUse;
    let correspondence = receipt.output_correspondence();
    if correspondence.binding_type() != Some(TypeId::of::<Binding>()) {
        return Err(mismatch("foreign birth output binding"));
    }
    correspondence
        .bound_entity(&OutputRoleUse::fixed::<Role>())
        .map_err(|_| mismatch("foreign birth output role"))?
        .ok_or_else(|| mismatch("birth subject is absent from receipt"))
}

impl WorthQueryWorkflowBirthAdapter {
    pub const fn bind_idempotency(
        idempotency: WorthQueryApplicationIdempotencyBinding,
        identity: &[u8; 32],
    ) -> WorthQueryApplicationIdempotencyBinding {
        idempotency.bind_workflow_instance(identity)
    }
}
