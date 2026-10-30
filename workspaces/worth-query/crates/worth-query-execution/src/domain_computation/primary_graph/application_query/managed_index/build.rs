//! One finite provider build from an actual completed source's owned rows.
use super::source_rows::SourceImageView;
use super::{
    invocation::{decoded_provider, provider_result},
    WorthQueryManagedIndexNodeWriter, WorthQueryManagedIndexNodes, WorthQueryManagedIndexProvider,
    WorthQueryManagedIndexSourceRows, WorthQueryManagedIndexWorkDenial,
};
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};

pub(super) fn build<S, I, P>(
    execution: &ExecutionRequest<'_, '_>,
    provider: &P,
    build_session: WorthQueryGraphWorkSessionIdentity,
    image: SourceImageView<'_>,
) -> Result<WorthQueryManagedIndexNodes<P::Node>, WorthQueryManagedIndexWorkDenial>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    decoded_provider(execution, image.bindings, |control, session| {
        let sources = WorthQueryManagedIndexSourceRows::<S, I>::new(image, session, control);
        let capacity = provider_result(
            execution,
            control,
            provider.node_capacity(sources.len(), control),
        )?;
        let mut writer = WorthQueryManagedIndexNodeWriter::allocate(
            execution,
            build_session,
            capacity,
            sources.len(),
            control,
        )?;
        let result = provider.build(&sources, &mut writer, control);
        sources.check()?;
        let root = provider_result(execution, control, result)?;
        writer.seal(execution, root).map_err(Into::into)
    })
}
