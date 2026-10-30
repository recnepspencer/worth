pub use worth_query_testkit_installed_operation_fixture::facade::query::support;

pub use support::installed_domain::{workspace, PublicInstalledDomain};
pub use worth_query::facade::{domain, runtime};

pub const READ_CAPABILITIES: &[domain::WorthQueryCapabilityFamily] =
    &[domain::WorthQueryCapabilityFamily::QueryRead];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConsumerOperatingContext;

impl domain::WorthQueryDomainOperatingContext<PublicInstalledDomain> for ConsumerOperatingContext {
    fn required_capability_families(&self) -> &'static [domain::WorthQueryCapabilityFamily] {
        READ_CAPABILITIES
    }

    fn required_config_sections(&self) -> &'static [domain::WorthQueryConfigSectionFamily] {
        &[domain::WorthQueryConfigSectionFamily::Query]
    }

    fn context_identity(&self) -> domain::WorthQueryDomainOperatingContextIdentityDeclaration {
        domain::WorthQueryDomainOperatingContextIdentityDeclaration::single("consumer-context:read")
    }
}

pub trait ConsumerDomainQueryExt {
    fn consumer_context(
        &self,
        workspace: &runtime::WorthQueryWorkspace,
    ) -> Result<
        domain::WorthQueryInstalledDomainDeclarationContext<
            PublicInstalledDomain,
            ConsumerOperatingContext,
        >,
        domain::WorthQueryInstalledDomainDeclarationContextDenial,
    >;
}

impl ConsumerDomainQueryExt for domain::WorthQueryInstalledDomainHandle<PublicInstalledDomain> {
    fn consumer_context(
        &self,
        workspace: &runtime::WorthQueryWorkspace,
    ) -> Result<
        domain::WorthQueryInstalledDomainDeclarationContext<
            PublicInstalledDomain,
            ConsumerOperatingContext,
        >,
        domain::WorthQueryInstalledDomainDeclarationContextDenial,
    > {
        self.declarations_in(workspace, ConsumerOperatingContext)
    }
}
