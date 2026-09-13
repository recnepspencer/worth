mod bridge_registrations;
mod graph_participation;
mod installed_operation;
mod runtime_resources;

use worth_query::facade::{domain, runtime};

pub(crate) use bridge_registrations::presentation_bridge_registrations;
use graph_participation::{
    presentation_graph_definition, WorthUiPresentationGraphProvider,
    WorthUiPresentationSemanticGraph,
};
pub(crate) use installed_operation::WorthUiPresentationAsyncDomainEntry;
use installed_operation::{
    presentation_aspect_contracts, presentation_async_definition,
    WorthUiPresentationAsyncOperationExecutor,
};
pub(super) use installed_operation::{
    WorthUiPresentationAsyncOperation, WorthUiPresentationAsyncOperationFamily,
};

pub(crate) fn worth_ui_presentation_async_domain_package(
) -> domain::WorthQueryDomainPackage<WorthUiPresentationAsyncDomainEntry> {
    domain::WorthQueryDomainPackage::declare(
        WorthUiPresentationAsyncDomainEntry,
        domain::WorthQueryDomainIdentityDeclaration::new(
            domain::WorthQueryDomainIdentityNamespace::new("WORTH.ui")
                .expect("static WUI namespace must admit"),
            domain::WorthQueryDomainIdentityName::new("presentation-async")
                .expect("static WUI presentation domain must admit"),
            domain::WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .requires_capability(domain::WorthQueryCapabilityFamily::QueryRead)
    .requires_capability(domain::WorthQueryCapabilityFamily::QueryComposition)
    .requires_configuration(domain::WorthQueryConfigSectionFamily::Query)
    .requires_configuration(domain::WorthQueryConfigSectionFamily::Relational)
    .operation(presentation_async_definition())
    .operation_graph_participation::<
        WorthUiPresentationAsyncOperation,
        WorthUiPresentationAsyncOperationFamily,
        WorthUiPresentationSemanticGraph,
    >("presentation")
}

pub(crate) fn install_worth_ui_presentation_async_runtime(
    builder: runtime::WorthQueryRuntimeBuilder,
) -> Result<runtime::WorthQueryRuntimeBuilder, runtime::WorthQueryAspectContractRegistrationDenial>
{
    let builder = builder.aspect_contracts(presentation_aspect_contracts())?;
    Ok(builder
        .conditional_execution_resources(runtime_resources::presentation_async_resources())
        .owned_bridge_async_declaration(
            super::runtime_bridge::presentation_owned_async_source_declaration(),
        )
        .graph_participation(presentation_graph_definition())
        .graph_participation_provider(
            WorthUiPresentationSemanticGraph,
            WorthUiPresentationGraphProvider,
        )
        .domain_operation_executor(
            WorthUiPresentationAsyncDomainEntry,
            WorthUiPresentationAsyncOperation,
            WorthUiPresentationAsyncOperationFamily,
            WorthUiPresentationAsyncOperationExecutor,
        ))
}
