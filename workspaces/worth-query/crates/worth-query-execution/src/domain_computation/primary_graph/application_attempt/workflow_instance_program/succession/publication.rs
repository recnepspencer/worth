//! Descriptive projection of the exact native successor recorded by this request.
use super::super::super::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationIdempotencyBinding,
};
use super::super::PublishedWorkflowInstanceRef;
use super::{
    mismatch, RequiredWorkflowSuccession, WorthQueryApplicationAttemptDenial,
    WorthQueryWorkflowSuccessionAdapter,
};
impl RequiredWorkflowSuccession {
    pub fn instance_in(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<PublishedWorkflowInstanceRef, WorthQueryApplicationAttemptDenial> {
        let idempotency = receipt.idempotency_binding();
        if receipt.product_branch() != self.branch
            || receipt.principal_scope() != &self.scope
            || idempotency.key_identity() != &self.key_identity
            || idempotency.intent_identity() != &self.input_identity
            || !idempotency.matches_workflow_instance(&self.identity)
            || !idempotency.is_for_mutation_binding(self.binding)
        {
            return Err(mismatch(
                "receipt does not describe this succession request",
            ));
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
        .ok_or_else(|| mismatch("receipt does not record exactly one native successor"))
    }
}
pub(super) fn subject_in_receipt<Schema, Binding, Role>(
    receipt: &WorthQueryApplicationCommitReceipt,
) -> Result<worth_relational::facade::identity::EntityId, WorthQueryApplicationAttemptDenial>
where
    Schema: super::ApplicationSchema,
    Binding: super::ApplicationMutationBinding<Schema>,
    Role: super::WorthQueryApplicationOutputRole<Schema = Schema, Contract = Binding::Output>,
{
    use crate::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::OutputRoleUse;
    let correspondence = receipt.output_correspondence();
    if correspondence.binding_type() != Some(super::TypeId::of::<Binding>()) {
        return Err(mismatch("foreign succession output binding"));
    }
    correspondence
        .bound_entity(&OutputRoleUse::fixed::<Role>())
        .map_err(|_| mismatch("foreign succession output role"))?
        .ok_or_else(|| mismatch("succession subject absent from receipt"))
}
impl WorthQueryWorkflowSuccessionAdapter {
    pub const fn bind_idempotency(
        idempotency: WorthQueryApplicationIdempotencyBinding,
        identity: &[u8; 32],
    ) -> WorthQueryApplicationIdempotencyBinding {
        idempotency.bind_workflow_instance(identity)
    }
}
