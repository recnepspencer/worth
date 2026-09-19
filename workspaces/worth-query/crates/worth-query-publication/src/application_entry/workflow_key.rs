use sha2::{Digest, Sha256};
use worth_foundational::facade::prepare_aspect_value_identity_basis;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityWorkflowIdempotency,
    application_schema::{
        ApplicationIdentityScalarValueBinding, ApplicationOperationMarkerIdentity,
        ApplicationValueEncodeDenial,
    },
};
use worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyBinding;

pub(super) fn workflow_idempotency<
    Schema,
    Operation,
    Input,
    Key,
    PrincipalIdentity,
    PrincipalIdentityBinding,
>(
    key: &Key,
    input: &Input,
    principal: &PrincipalIdentity,
) -> Result<WorthQueryApplicationIdempotencyBinding, ApplicationValueEncodeDenial>
where
    Operation: ApplicationOperationMarkerIdentity<Schema>
        + ApplicationCapabilityWorkflowIdempotency<Schema, Input, Key>,
    PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
{
    let principal_value = PrincipalIdentityBinding::encode(principal)?;
    let principal_basis = prepare_aspect_value_identity_basis(&principal_value);
    let mut digest = Sha256::new();
    for part in [
        b"worth-query.capability-workflow-key.v1".as_slice(),
        Operation::IDENTIFIER.as_bytes(),
        principal_basis.as_str().as_bytes(),
        Operation::client_key_identity(key).as_slice(),
    ] {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part);
    }
    Ok(WorthQueryApplicationIdempotencyBinding::new(
        digest.finalize().into(),
        Operation::input_identity(input),
    ))
}
