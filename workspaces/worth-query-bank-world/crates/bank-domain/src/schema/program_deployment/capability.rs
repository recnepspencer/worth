//! The current source program selects the exact deployment resource.
use super::*;
use crate::model::BankPrincipalId;
use crate::schema::{BankPrincipalBinding, BankPrincipalIdBinding, ExternalPrincipalMapping};
use worth_query_decl::facade::{
    application_capability::*, application_program::*, worth_query_capability,
    worth_query_capability_context, worth_query_capability_provenance, worth_query_operation,
    worth_query_operation_reads, worth_query_portable_type, worth_query_structured_value_binding,
};
worth_query_capability!(pub DeployBankProgram in BankSchema, identity "worth.bank.program-deployment-capability.v1");
worth_query_capability_context!(pub ProgramDeploymentContext in BankSchema);
worth_query_capability_provenance!(pub ProgramDeploymentProvenance in BankSchema);
#[derive(serde::Serialize)]
pub struct ProgramDeploymentInput {
    source: String,
    target: String,
}
worth_query_portable_type!(ProgramDeploymentInput => "worth.bank.program-deployment-input.v1");
worth_query_structured_value_binding!(pub ProgramDeploymentInputBinding for ProgramDeploymentInput { identity: "worth.bank.program-deployment-input.v1" });
worth_query_operation!(pub DeployBankProgramOperation for BankSchema, input ProgramDeploymentInputBinding);
worth_query_operation_reads!(DeployBankProgramOperation => [ProgramDeploymentSourceRevision]);
pub struct BankProgramDeploymentBinding;
impl ApplicationProgramAdoptionBinding<BankSchema> for BankProgramDeploymentBinding {
    const IDENTITY: &'static str = "worth.bank.program-deployment-binding.v1";
    type Capability = DeployBankProgram;
    type Input = ProgramDeploymentInput;
    type InputBinding = ProgramDeploymentInputBinding;
    type Operation = DeployBankProgramOperation;
    type PrincipalBinding = BankPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = BankPrincipalId;
    type PrincipalIdentityBinding = BankPrincipalIdBinding;
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        BankSchema,
        BankPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        BankPrincipalId,
        BankPrincipalIdBinding,
    > {
        BankPrincipalBinding::reference()
    }
    fn input(context: ApplicationProgramAdoptionContext<'_>) -> ProgramDeploymentInput {
        ProgramDeploymentInput {
            source: context.source.to_string(),
            target: context.target.to_string(),
        }
    }
}
impl ApplicationCapabilityRequest<BankSchema, DeployBankProgram> for ProgramDeploymentInput {
    type Scope = ProgramDeploymentScope;
    type Context = ProgramDeploymentContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<BankSchema, Self::Scope, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                ProgramDeploymentSourceRevision::reference(),
                encoded(self.source.clone()),
            ),
            encoded("deploy-bank-program".to_owned()),
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
