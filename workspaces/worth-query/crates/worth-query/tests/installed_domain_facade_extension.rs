use crate::support;

use support::installed_domain::{workspace, PublicInstalledDomain};
use worth_query::facade::{domain, runtime};

const READ_CAPABILITIES: &[domain::WorthQueryCapabilityFamily] =
    &[domain::WorthQueryCapabilityFamily::QueryRead];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ConsumerOperatingContext;

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

trait ConsumerDomainQueryExt {
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

#[test]
fn downstream_extension_preserves_the_generic_installed_capability_artifact() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = workspace(request_execution, "installed-domain-facade-extension");
    let handle = workspace
        .domain(PublicInstalledDomain)
        .expect("one installed handle lookup should admit");

    let generic = handle
        .declarations_in(&workspace, ConsumerOperatingContext)
        .expect("generic installed context should admit");
    let extended = handle
        .consumer_context(&workspace)
        .expect("downstream extension should lower through the same handle");

    assert_eq!(generic.handle_identity(), extended.handle_identity());
    assert_eq!(
        generic.installed_authority().witness_identity(),
        extended.installed_authority().witness_identity()
    );

    let runtime = workspace.into_runtime();
    let receipt = runtime
        .domain_installation_receipt(PublicInstalledDomain)
        .expect("the one package installation should retain a receipt");
    assert_eq!(receipt.construction_counters().package_lowerings(), 1);
    assert_eq!(receipt.construction_counters().derived_index_builds(), 1);
    assert_eq!(
        runtime
            .domain_installation_lookup_counters()
            .handle_lookups(),
        1
    );
    assert_eq!(
        runtime
            .domain_installation_lookup_counters()
            .package_content_scans(),
        0
    );
}
