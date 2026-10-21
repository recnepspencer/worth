//! Assessment uses the same checked source-expectation owner as execution.
use super::*;
use crate::application_entry::mutation::request::WorthQueryMutationExpectedSource;
use worth_query_execution::facade::runtime::ExecutionAllocationPolicy;

pub(super) fn bind<Schema, Intent, Preparation>(
    request: &mut WorthQueryApplicationMutationRequest<'_, '_, '_, Schema, Intent, Preparation>,
    admission: &mut Admission<Schema, Intent>,
) -> Result<(), WorthQueryApplicationRequestMutationDenial>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    let Some(source) = request.source.take() else {
        return Ok(());
    };
    let pending = match source {
        WorthQueryMutationExpectedSource::Row(source) => request
            .application
            .bind_application_source_expectation::<Intent::Binding, _>(
                admission,
                source,
                request.intent.input(),
            ),
        WorthQueryMutationExpectedSource::ResultSet(source) => request
            .application
            .bind_application_result_set_expectation::<Intent::Binding, _>(
                admission,
                source,
                request.intent.input(),
            ),
    }
    .map_err(WorthQueryApplicationRequestMutationDenial::SourceExpectation)?;
    pending
        .consume_into(
            request.application,
            admission,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .map_err(WorthQueryApplicationRequestMutationDenial::SourceExpectation)
}
