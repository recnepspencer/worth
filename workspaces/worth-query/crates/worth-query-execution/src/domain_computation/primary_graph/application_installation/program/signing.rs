//! One immutable signing service per installed program revision, not per workflow frontier.
use super::WorthQueryProgramApplicationRuntime;
use worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventSigningOwner;
use worth_query_declaration::facade::{
    application_program::ApplicationProgramRevision, application_schema::ApplicationSchema,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQuerySigningOwnerInstallationDenial {
    ForeignSchema,
    ProgramUnavailable,
    AlreadyInstalled,
}
impl std::fmt::Display for WorthQuerySigningOwnerInstallationDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "signing owner installation denied: {self:?}")
    }
}
impl std::error::Error for WorthQuerySigningOwnerInstallationDenial {}

impl<Schema: ApplicationSchema, Program> WorthQueryProgramApplicationRuntime<Schema, Program> {
    /// Installs once for an exact rostered revision; replacement is never allowed.
    pub fn install_signing_owner(
        &mut self,
        revision: ApplicationProgramRevision,
        owner: WorthQueryAuthenticationEventSigningOwner<Schema>,
    ) -> Result<(), WorthQuerySigningOwnerInstallationDenial> {
        validate(self.runtime(), &revision, &owner)?;
        AdmittedSigningOwner::admit(&self.runtime().signing_owners, revision, owner)?.install();
        Ok(())
    }
}

pub(super) fn validate<Schema: ApplicationSchema>(
    runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
        Schema,
    >,
    revision: &ApplicationProgramRevision,
    owner: &WorthQueryAuthenticationEventSigningOwner<Schema>,
) -> Result<(), WorthQuerySigningOwnerInstallationDenial> {
    if owner.binding_identity() != &runtime.installed_schema.binding_identity() {
        return Err(WorthQuerySigningOwnerInstallationDenial::ForeignSchema);
    }
    if runtime
        .installed_program_support()
        .and_then(|support| support.present(revision))
        .is_none()
    {
        return Err(WorthQuerySigningOwnerInstallationDenial::ProgramUnavailable);
    }
    Ok(())
}

pub(super) struct AdmittedSigningOwner<'a, Schema> {
    owners: std::sync::MutexGuard<
        'a,
        std::collections::BTreeMap<
            ApplicationProgramRevision,
            WorthQueryAuthenticationEventSigningOwner<Schema>,
        >,
    >,
    revision: ApplicationProgramRevision,
    owner: WorthQueryAuthenticationEventSigningOwner<Schema>,
}
impl<'a, Schema> AdmittedSigningOwner<'a, Schema> {
    pub(super) fn admit(
        owners: &'a std::sync::Mutex<
            std::collections::BTreeMap<
                ApplicationProgramRevision,
                WorthQueryAuthenticationEventSigningOwner<Schema>,
            >,
        >,
        revision: ApplicationProgramRevision,
        owner: WorthQueryAuthenticationEventSigningOwner<Schema>,
    ) -> Result<Self, WorthQuerySigningOwnerInstallationDenial> {
        let owners = owners
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(installed) = owners.get(&revision)
            && !installed.same_owner(&owner)
        {
            return Err(WorthQuerySigningOwnerInstallationDenial::AlreadyInstalled);
        }
        Ok(Self {
            owners,
            revision,
            owner,
        })
    }
    pub(super) fn install(mut self) {
        self.owners.entry(self.revision).or_insert(self.owner);
    }
}
