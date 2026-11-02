//! Shared ordinary attachment and request construction for the real consumer.
use super::*;

pub(crate) fn with_request<
    P: worth_query_decl::facade::application_program::ApplicationProgramDefinition<ConsumerSchema>,
    T,
>(
    application: &installation::ConsumerRuntime<P>,
    f: impl FnOnce(&Request<'_, '_, '_>) -> T,
) -> T {
    with_branch_request(application, application.current_world(), f)
}
pub(crate) fn with_branch_request<
    P: worth_query_decl::facade::application_program::ApplicationProgramDefinition<ConsumerSchema>,
    T,
>(
    application: &installation::ConsumerRuntime<P>,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    f: impl FnOnce(&Request<'_, '_, '_>) -> T,
) -> T {
    let adapter = authentication::admit(application.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    f(&application.request(&principal, &scope).on_branch(branch))
}
pub(crate) fn attach<
    'a,
    P: worth_query_decl::facade::application_program::ApplicationProgramDefinition<ConsumerSchema>,
>(
    application: &'a installation::ConsumerRuntime<P>,
    key: &str,
) -> Document<'a> {
    attach_with_branch(application, key).0
}
pub(crate) fn attach_with_branch<
    'a,
    P: worth_query_decl::facade::application_program::ApplicationProgramDefinition<ConsumerSchema>,
>(
    application: &'a installation::ConsumerRuntime<P>,
    key: &str,
) -> (
    Document<'a>,
    worth_query_host::facade::product::WorthQueryProductBranch,
) {
    let runtime = application.runtime();
    application
        .with_host_advancement(|phase| {
            let selected = runtime
                .on_branch(application.current_world())
                .select(&phase)
                .unwrap();
            let candidate = runtime
                .branches()
                .fork_expected(selected)
                .unwrap()
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap();
            let head = candidate.into_owned_head();
            let branch = head.branch();
            let document = with_branch_request(application, branch, |request| {
                request
                    .attach_history::<History>(
                        &phase,
                        head,
                        PlanarOutputRead {
                            body_key: key.into(),
                        },
                    )
                    .unwrap_or_else(|failure| {
                        let (_, cause) = failure.into_parts();
                        panic!("history attach: {cause:?}")
                    })
            });
            (document, branch)
        })
        .unwrap()
}
pub(crate) fn close<P, O>(document: Document<'_, O>, application: &installation::ConsumerRuntime<P>)
where
    P: worth_query_decl::facade::application_program::ApplicationProgramDefinition<ConsumerSchema>,
    O: WorthQueryHistoryOutputsMode<ConsumerSchema, History>,
{
    application
        .with_host_advancement(|phase| {
            document
                .close(&phase)
                .unwrap_or_else(|failure| panic!("history close: {failure:?}"))
        })
        .unwrap();
}
