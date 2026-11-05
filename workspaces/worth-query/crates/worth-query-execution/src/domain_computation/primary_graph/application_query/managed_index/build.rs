//! One finite provider build from an actual completed source's owned rows.
use super::source_rows::SourceImageView;
use crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity;
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use {
    super::invocation::decoded_provider, super::invocation::provider_result,
    super::WorthQueryManagedIndexNodeWriter, super::WorthQueryManagedIndexNodes,
    super::WorthQueryManagedIndexProvider, super::WorthQueryManagedIndexSourceRows,
    crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial,
};

pub(super) fn build<S, I, P>(
    execution: &ExecutionRequest<'_, '_>,
    provider: &P,
    build_session: WorthQueryGraphWorkSessionIdentity,
    image: SourceImageView<'_>,
) -> Result<
    (
        WorthQueryManagedIndexNodes<P::Node>,
        Option<super::source_structure::SourceTree>,
    ),
    WorthQueryManagedIndexWorkDenial,
>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    decoded_provider(execution, image.bindings, |control, session| {
        let source_control = image
            .source_budget
            .map(|budget| control.under_source(budget));
        let control = source_control.as_ref().unwrap_or(control);
        let sources =
            WorthQueryManagedIndexSourceRows::<S, I>::new(image, session, control, *execution);
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
        let mut nodes = writer.seal(execution, root)?;
        let structure = super::source_structure::qualify(
            execution,
            provider,
            &sources,
            &mut nodes,
            super::source_structure::Step::Cold,
            control,
        )?;
        Ok((nodes, structure))
    })
}
