//! Descriptive projection of the exact native successor recorded by this request.
use super::super::super::{
    WorthQueryApplicationCommitReceipt,
};
use crate::domain_computation::primary_graph::vocabulary::application_attempt::workflow_instance_program_publication::PublishedWorkflowInstanceRef;
use super::{
    mismatch, RequiredWorkflowSuccession, WorthQueryApplicationAttemptDenial,
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
    use crate::domain_computation::primary_graph::vocabulary::application_attempt::role_use::OutputRoleUse;
    let correspondence = receipt.output_correspondence();
    if correspondence.binding_type() != Some(super::TypeId::of::<Binding>()) {
        return Err(mismatch("foreign succession output binding"));
    }
    correspondence
        .bound_entity(&OutputRoleUse::fixed::<Role>())
        .map_err(|_| mismatch("foreign succession output role"))?
        .ok_or_else(|| mismatch("succession subject absent from receipt"))
}
