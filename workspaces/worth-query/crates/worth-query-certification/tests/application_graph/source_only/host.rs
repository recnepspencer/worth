//! Installation through the existing native contribution and bootstrap owners.
use super::{
    fixture::SourceOnlyRetentionProgram,
    handler::{Contacts, RetentionHandler},
    schema::*,
};
use std::sync::Arc;
use worth_query_host::facade::{
    application_contribution::*, application_installation::*, declaration::application_program::*,
    primary_graph::*,
};
pub struct Host {
    runtime:
        WorthQueryProgramApplicationRuntime<SourceOnlyRetentionSchema, SourceOnlyRetentionProgram>,
    pub contacts: Arc<Contacts>,
}
impl std::ops::Deref for Host {
    type Target =
        WorthQueryProgramApplicationRuntime<SourceOnlyRetentionSchema, SourceOnlyRetentionProgram>;
    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}
impl WorthQueryApplicationContribution<SourceOnlyRetentionSchema>
    for SourceOnlyRetentionContribution
{
    type Configuration = Arc<Contacts>;
    fn configure(
        contacts: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, SourceOnlyRetentionSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<SetRetentionBinding, _>(RetentionHandler(contacts))
    }
}
pub fn install() -> Host {
    let contacts = Arc::new(Contacts::default());
    let runtime = in_memory_rostered_program(
        ApplicationProgramAuthoring::<SourceOnlyRetentionSchema, SourceOnlyRetentionProgram>::begin().validated_program().unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        SourceOnlyRetentionSchema::declaration().unwrap(),
        (contacts.clone(),),
        crate::document_retention_model::host::host_limits(),
        |_, graph, installed| {
            let binding = installed.principal_binding(DocumentPrincipalBinding::reference()).unwrap();
            graph.bind_principal(&binding, WorthQueryApplicationPrincipalKey::new("source-only-operator").unwrap(), 1_u64, crate::document_retention_model::operator_identity::operator_identity(), worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled)?;
            graph.bind_entity(WorthQueryApplicationEntitySeed::new(Document::reference(), WorthQueryApplicationEntityKey::new("document-row-1").unwrap()).field(DocumentIdentityField::reference(), DOCUMENT_IDENTITY.to_owned()).field(DocumentRetentionField::reference(), 7_u64))?;
            Ok(())
        },
    ).unwrap();
    Host { runtime, contacts }
}
