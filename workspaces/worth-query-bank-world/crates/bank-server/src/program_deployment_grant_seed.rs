//! Explicit deployment grants supplied by the Bank installation owner.
use bank_domain::{
    model::BankPrincipalId,
    schema::{BankSchema, ProgramDeploymentGrant},
};
use worth_query_host::facade::{
    declaration::application_program::{
        ApplicationProgramAuthoring, ApplicationProgramDefinition, ApplicationProgramRevision,
        ApplicationProgramValidationDenial,
    },
    primary_graph::WorthQueryApplicationEntityKey,
};
/// One explicit actor's permission to deploy from one exact source revision.
/// Supplying this seed does not grant permissions for later source revisions.
pub struct BankProgramDeploymentGrantSeed {
    pub(crate) key: WorthQueryApplicationEntityKey<BankSchema, ProgramDeploymentGrant>,
    pub(crate) source: ApplicationProgramRevision,
    pub(crate) grantor: BankPrincipalId,
    pub(crate) grantee: BankPrincipalId,
    pub(crate) not_before: u64,
    pub(crate) not_after: u64,
}
/// Invalid deployment grant setup; no runtime is installed from it.
#[derive(Debug)]
pub enum BankProgramDeploymentGrantSeedDenial {
    InvalidIdentity,
    InvalidValidity,
    Program(ApplicationProgramValidationDenial),
}
impl BankProgramDeploymentGrantSeed {
    /// Uses the canonical revision produced by the declared source program.
    pub fn for_program<Source: ApplicationProgramDefinition<BankSchema>>(
        identity: impl Into<String>,
        grantor: BankPrincipalId,
        grantee: BankPrincipalId,
        not_before: u64,
        not_after: u64,
    ) -> Result<Self, BankProgramDeploymentGrantSeedDenial> {
        let identity = identity.into();
        let key = WorthQueryApplicationEntityKey::new(identity.clone())
            .map_err(|_| BankProgramDeploymentGrantSeedDenial::InvalidIdentity)?;
        if not_before > not_after {
            return Err(BankProgramDeploymentGrantSeedDenial::InvalidValidity);
        }
        let source = *ApplicationProgramAuthoring::<BankSchema, Source>::begin()
            .validated_program()
            .map_err(BankProgramDeploymentGrantSeedDenial::Program)?
            .revision();
        Ok(Self {
            key,
            source,
            grantor,
            grantee,
            not_before,
            not_after,
        })
    }
}
