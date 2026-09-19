use worth_query_declaration::facade::application_operation::ApplicationMutationIntent;
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
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationDiscoveredOutputConnection, WorthQueryApplicationProjection,
};

use super::{
    Discovery, RootConnection, WorthQueryPerformedDiscoveredApplicationMutation,
    WorthQueryStartedDiscoveredOutputs,
};
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

pub struct WorthQueryDiscoveredOutputStartFailure<'application, Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
{
    performed: WorthQueryPerformedDiscoveredApplicationMutation<
        'application,
        Schema,
        Intent,
        Program,
        Root,
    >,
    denial: WorthQueryRequiredOutputPreparationDenial,
}

impl<'application, Schema, Intent, Program, Root>
    WorthQueryDiscoveredOutputStartFailure<'application, Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
{
    pub fn performed(
        self,
    ) -> WorthQueryPerformedDiscoveredApplicationMutation<'application, Schema, Intent, Program, Root>
    {
        self.performed
    }

    pub const fn denial(&self) -> &WorthQueryRequiredOutputPreparationDenial {
        &self.denial
    }
}

impl<'application, Schema, Intent, Program, Root>
    WorthQueryPerformedDiscoveredApplicationMutation<'application, Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
    Root::Dependents:
        ProgramOutputContinuationFactory<'application, Schema, Program, RootDemand<Schema, Root>>,
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
    pub fn start_required_outputs(
        self,
        request: &WorthQueryApplicationRequest<'application, '_, '_, Schema>,
        controls: WorthQueryOutputDemandControls,
    ) -> Result<
        WorthQueryStartedDiscoveredOutputs<'application, Schema, Intent, Program, Root>,
        WorthQueryDiscoveredOutputStartFailure<'application, Schema, Intent, Program, Root>,
    > {
        if let Err(denial) = self
            .application
            .validate_program_discovered_root_artifact_source::<Root, Intent::Binding>(
                &worth_query_execution::publication_boundary::program_publication_access(),
            )
        {
            return Err(WorthQueryDiscoveredOutputStartFailure {
                performed: self,
                denial: WorthQueryRequiredOutputPreparationDenial::Demand(
                    crate::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(
                        denial,
                    ),
                ),
            });
        }
        let retained =
            WorthQueryApplicationReadObservation::new(std::sync::Arc::clone(&self.retained_source));
        let required_output = match super::resolve::start_discovered_roots::<Schema, Program, Root>(
            self.application,
            request,
            &self.receipt,
            self.discovery.clone(),
            retained,
            &self.prepared,
            controls,
            super::resolve::DiscoveredRootStartKind::Performed,
        ) {
            Ok(required_output) => required_output,
            Err(denial) => {
                return Err(WorthQueryDiscoveredOutputStartFailure {
                    performed: self,
                    denial,
                })
            }
        };
        Ok(WorthQueryStartedDiscoveredOutputs {
            receipt: self.receipt,
            result: self.result,
            required_output,
        })
    }
}
