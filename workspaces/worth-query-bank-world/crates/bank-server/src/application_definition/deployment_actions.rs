//! Deployment revocation retains its recorded-issuer policy and original u64 key.
use super::composition::BankAccountsFeature;
use bank_domain::schema::{BankSchema, ProgramDeploymentRevocationBinding};
use worth_query_host::facade::declaration::application_program::{
    ApplicationActionDelivery, ApplicationActionReference, ApplicationRootComposition,
    ApplicationSourceCompletion,
};
/// Source completion for recorded-issuer deployment grant revocation.
pub struct ProgramDeploymentRevocationDelivery;
impl ApplicationActionDelivery<BankSchema, ProgramDeploymentRevocationBinding>
    for ProgramDeploymentRevocationDelivery
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "bank.program-deployment.revocation.completion.v1";
}
/// The exact installed deployment revocation action.
pub type ProgramDeploymentRevocationAction = ApplicationActionReference<
    BankSchema,
    ApplicationRootComposition,
    BankAccountsFeature,
    ProgramDeploymentRevocationBinding,
    ProgramDeploymentRevocationDelivery,
>;
