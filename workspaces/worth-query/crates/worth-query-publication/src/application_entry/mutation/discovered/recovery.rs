use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationStructuredValueBinding,
};
use worth_query_execution::facade::application_contribution::{
    WorthQueryApplicationOutputDemand, WorthQueryProducerOutputFamily,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationDiscoveredOutputConnection,
    WorthQueryApplicationProjection,
};

use super::{Discovery, RootConnection, WorthQueryDiscoveredProgramOutputHandle};
use crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory;
use crate::application_entry::{
    WorthQueryApplicationReadObservation, WorthQueryApplicationRequest,
    WorthQueryOutputDemandControls, WorthQueryRequiredOutputPreparationDenial,
};

type DiscoveryBinding<Schema, Root> =
    <Discovery<Schema, Root> as ApplicationQueryIntent<Schema>>::Binding;
type DiscoveryQuery<Schema, Root> =
    <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::Query;
type DiscoveryValue<Schema, Root> = <<DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<
    Schema,
>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type RootDemand<Schema, Root> =
    <RootConnection<Schema, Root> as WorthQueryApplicationDiscoveredOutputConnection<Schema>>::Demand;
type RootFamily<Schema, Root> =
    <RootDemand<Schema, Root> as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type RootSource<Schema, Root> =
    <RootFamily<Schema, Root> as WorthQueryProducerOutputFamily<Schema>>::Source;
type RootSourceQuery<Schema, Root> =
    <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Query;
type RootSourceValue<Schema, Root> = <<RootSource<Schema, Root> as ApplicationQueryBinding<
    Schema,
>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

impl<'application, Schema> WorthQueryApplicationRequest<'application, '_, '_, Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub fn recover_discovered_required_outputs<Program, Root>(
        &self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        receipt: &WorthQueryApplicationCommitReceipt,
        controls: WorthQueryOutputDemandControls,
    ) -> Result<
        WorthQueryDiscoveredProgramOutputHandle<'application, Schema, Program, Root>,
        WorthQueryRequiredOutputPreparationDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
        Root::Dependents: ProgramOutputContinuationFactory<
            'application, Schema, Program, RootDemand<Schema, Root>,
        >,
        RootDemand<Schema, Root>: Clone,
        DiscoveryValue<Schema, Root>:
            WorthQueryApplicationProjection<Schema, DiscoveryQuery<Schema, Root>> + Clone,
        <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        RootSourceValue<Schema, Root>:
            WorthQueryApplicationProjection<Schema, RootSourceQuery<Schema, Root>> + Clone,
        <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Input:
            ApplicationQueryIntent<Schema, Binding = RootSource<Schema, Root>>,
        <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
    {
        if !std::ptr::eq(application.runtime(), self.application) {
            return Err(WorthQueryRequiredOutputPreparationDenial::ForeignProgram);
        }
        let (discovery, retained, prepared) = application
            .recover_discovered_program_source::<Root>(
                &worth_query_execution::publication_boundary::program_publication_access(),
                receipt,
            )
            .map_err(WorthQueryRequiredOutputPreparationDenial::DemandExecution)?;
        super::resolve::start_discovered_roots::<Schema, Program, Root>(
            application,
            self,
            receipt,
            discovery,
            WorthQueryApplicationReadObservation::new(retained),
            &prepared,
            controls,
            super::resolve::DiscoveredRootStartKind::Recovery,
        )
    }
}
