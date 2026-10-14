//! The current source program selects the exact deployment resource.
use super::*;

use worth_query_decl::facade::{
    application_capability::*, application_program::*, worth_query_capability,
    worth_query_capability_context, worth_query_capability_provenance, worth_query_operation,
    worth_query_operation_reads, worth_query_portable_type, worth_query_structured_value_binding,
};
use worth_query_topology_entry::{ConsumerPrincipalBinding, ExternalPrincipalMapping};
worth_query_capability!(pub DeployConsumerProgram in ConsumerSchema, identity "worth.query.certification.consumer.program-deployment-capability.v1");
worth_query_capability_context!(pub ProgramDeploymentContext in ConsumerSchema);
worth_query_capability_provenance!(pub ProgramDeploymentProvenance in ConsumerSchema);
#[derive(serde::Serialize)]
pub struct ProgramDeploymentInput {
    source: String,
    target: String,
}
worth_query_portable_type!(ProgramDeploymentInput => "worth.query.certification.consumer.program-deployment-input.v1");
worth_query_structured_value_binding!(pub ProgramDeploymentInputBinding for ProgramDeploymentInput { identity: "worth.query.certification.consumer.program-deployment-input.v1" });
worth_query_operation!(pub DeployConsumerProgramOperation for ConsumerSchema, input ProgramDeploymentInputBinding);
worth_query_operation_reads!(DeployConsumerProgramOperation => [ProgramDeploymentSourceRevision]);
pub struct ConsumerProgramDeploymentBinding;
impl ApplicationProgramAdoptionBinding<ConsumerSchema> for ConsumerProgramDeploymentBinding {
    const IDENTITY: &'static str =
        "worth.query.certification.consumer.program-deployment-binding.v1";
    type Capability = DeployConsumerProgram;
    type Input = ProgramDeploymentInput;
    type InputBinding = ProgramDeploymentInputBinding;
    type Operation = DeployConsumerProgramOperation;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        ConsumerSchema,
        ConsumerPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ConsumerPrincipalBinding::reference()
    }
    fn input(context: ApplicationProgramAdoptionContext<'_>) -> ProgramDeploymentInput {
        ProgramDeploymentInput {
            source: context.source.to_string(),
            target: context.target.to_string(),
        }
    }
}
impl ApplicationCapabilityRequest<ConsumerSchema, DeployConsumerProgram>
    for ProgramDeploymentInput
{
    type Scope = ProgramDeploymentScope;
    type Context = ProgramDeploymentContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<ConsumerSchema, Self::Scope, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                ProgramDeploymentSourceRevision::reference(),
                encoded(self.source.clone()),
            ),
            encoded("deploy-consumer-program".to_owned()),
            encoded("program-deployment".to_owned()),
            ApplicationCapabilityRequestContext::new(ProgramDeploymentContext::reference()),
        ))
    }
}
pub(super) fn encoded(
    value: String,
) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value).expect("deployment scalar is a string")
}
