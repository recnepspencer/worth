use worth_query::facade::{foundation, runtime};

const PRESENTATION_PAYLOAD_CONTRACT: u64 = 0x5755_4950_5245_5345;
const PRESENTATION_MAX_PAYLOAD_BYTES: u64 = 16 * 1024 * 1024;
const PRESENTATION_RETRY_MAX_ATTEMPTS: u32 = 3;
const PRESENTATION_RETRY_DELAY_TICKS: u64 = 2;
const PRESENTATION_TIMEOUT_TICKS: u64 = 5;

pub(crate) fn presentation_owned_async_source_declaration(
) -> runtime::WorthQueryOwnedAsyncRequestDeclaration {
    runtime::WorthQueryOwnedAsyncRequestDeclaration::from_async_resource_identity(
        presentation_owned_async_source_identity(),
        PRESENTATION_PAYLOAD_CONTRACT,
        PRESENTATION_MAX_PAYLOAD_BYTES,
        PRESENTATION_RETRY_MAX_ATTEMPTS,
        PRESENTATION_RETRY_DELAY_TICKS,
        PRESENTATION_TIMEOUT_TICKS,
    )
}

pub(in crate::presentation_async) fn installed_presentation_owned_async_source(
    workspace: &runtime::WorthQueryWorkspace,
) -> Option<runtime::WorthQueryInstalledOwnedAsyncDeclaration> {
    workspace.installed_owned_bridge_async_declaration(&presentation_owned_async_source_identity())
}

fn presentation_owned_async_source_identity() -> foundation::WorthQueryAsyncResourceRequestIdentity
{
    foundation::WorthQueryAsyncResourceRequestIdentity::declare(
        foundation::WorthQueryAsyncSourceFamily::HostResource,
        foundation::WorthQueryAsyncLoadingPosture::Blocking,
        foundation::WorthQueryAsyncFailurePosture::RetainStaleValue,
        vec![foundation::WorthQueryAsyncRequestIdentityPart::text(
            "source",
            "worth-ui-presentation",
        )],
    )
    .expect("static presentation async source identity must admit")
}
