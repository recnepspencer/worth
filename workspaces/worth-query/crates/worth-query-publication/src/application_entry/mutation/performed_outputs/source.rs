use crate::application_entry::{
    WorthQueryApplicationReadObservation, WorthQueryApplicationRequest,
};
use worth_query_declaration::facade::{
    application_program::{
        ApplicationConnectionShape, ApplicationOutputGraphShape, ApplicationProgramDefinition,
        ApplicationRequiredOutputRoot,
    },
    application_schema::ApplicationSchema,
};
use worth_query_execution::facade::{
    application_installation::WorthQueryProgramApplicationRuntime,
    primary_graph::{
        WorthQueryApplicationCommitReceipt, WorthQueryApplicationRequiredOutputConnection,
        WorthQueryOutputDemandDenial, WorthQueryPreparedRequiredOutputSource,
    },
};

pub(super) type RequiredSourceValidator<Schema, Program> =
    fn(
        &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        &WorthQueryProgramApplicationRuntime<Schema, Program>,
        &WorthQueryPreparedRequiredOutputSource,
        &WorthQueryApplicationCommitReceipt,
        &WorthQueryApplicationReadObservation,
        &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<(), WorthQueryOutputDemandDenial>;

pub(super) fn validate<Schema, Program, Root>(
    phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    prepared: &WorthQueryPreparedRequiredOutputSource,
    receipt: &WorthQueryApplicationCommitReceipt,
    observation: &WorthQueryApplicationReadObservation,
    request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
) -> Result<(), WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    <Root::RootConnection as ApplicationConnectionShape<Schema>>::Binding:
        WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    application.validate_required_program_source::<Root>(
        phase,
        &worth_query_execution::publication_boundary::program_publication_access(),
        prepared,
        receipt,
        &observation.retained,
        request.principal,
        request.scope,
        request.branch,
    )
}
