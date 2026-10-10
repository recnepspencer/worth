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

/// A landed mutation whose discovered outputs could not start. `performed` returns the
/// landed mutation.
pub struct WorthQueryDiscoveredOutputStartFailure<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
{
    performed: WorthQueryPerformedDiscoveredApplicationMutation<Schema, Intent, Program, Root>,
    denial: WorthQueryRequiredOutputPreparationDenial,
}

impl<Schema, Intent, Program, Root>
    WorthQueryDiscoveredOutputStartFailure<Schema, Intent, Program, Root>
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
    ) -> WorthQueryPerformedDiscoveredApplicationMutation<Schema, Intent, Program, Root> {
        self.performed
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryPerformedDiscoveredApplicationMutation<Schema, Intent, Program, Root>,
        WorthQueryRequiredOutputPreparationDenial,
    ) {
        (self.performed, self.denial)
    }

    pub const fn denial(&self) -> &WorthQueryRequiredOutputPreparationDenial {
        &self.denial
    }
}

impl<Schema, Intent, Program, Root>
    WorthQueryPerformedDiscoveredApplicationMutation<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
    Root::Dependents: ProgramOutputContinuationFactory<Schema, Program, RootDemand<Schema, Root>>,
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
        application: &worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        controls: WorthQueryOutputDemandControls,
    ) -> Result<
        WorthQueryStartedDiscoveredOutputs<Schema, Intent, Program, Root>,
        WorthQueryDiscoveredOutputStartFailure<Schema, Intent, Program, Root>,
    > {
        let mut retained = Some(self);
        match application
            .runtime()
            .with_application_advancement(request.scope, |phase| {
                retained
                    .take()
                    .expect("host call retains its performed facts")
                    .start_required_outputs_in_advancement(&phase, application, request, controls)
            }) {
            Ok(outcome) => outcome,
            Err(cause) => Err(WorthQueryDiscoveredOutputStartFailure {
                performed: retained.take().expect("refused request performs no work"),
                denial: WorthQueryRequiredOutputPreparationDenial::advancement(cause),
            }),
        }
    }

    fn start_required_outputs_in_advancement(
        self,
        _phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        application: &worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        controls: WorthQueryOutputDemandControls,
    ) -> Result<
        WorthQueryStartedDiscoveredOutputs<Schema, Intent, Program, Root>,
        WorthQueryDiscoveredOutputStartFailure<Schema, Intent, Program, Root>,
    > {
        if !std::ptr::eq(application.runtime(), request.application) {
            return Err(WorthQueryDiscoveredOutputStartFailure {
                performed: self,
                denial: WorthQueryRequiredOutputPreparationDenial::ForeignProgram,
            });
        }
        let validation = application
            .validate_program_discovered_root_artifact_source::<Root, Intent::Binding>(
                &worth_query_execution::publication_boundary::program_publication_access(),
            )
            .and_then(|()| {
                application.validate_discovered_program_source::<Root>(
                    &worth_query_execution::publication_boundary::program_publication_access(),
                    &self.prepared,
                    &self.receipt,
                    &self.retained_source,
                    request.principal,
                    request.scope,
                    request.branch,
                )
            });
        if let Err(denial) = validation {
            return Err(WorthQueryDiscoveredOutputStartFailure {
                performed: self,
                denial: WorthQueryRequiredOutputPreparationDenial::DemandExecution(denial),
            });
        }
        let required_output = super::WorthQueryDiscoveredProgramOutputHandle::new(
            self.receipt.clone(),
            self.discovery,
            self.prepared,
            WorthQueryApplicationReadObservation::new(self.retained_source),
            controls,
            super::DiscoveredRootStartKind::Performed,
        );
        Ok(WorthQueryStartedDiscoveredOutputs {
            receipt: self.receipt,
            result: self.result,
            required_output,
        })
    }
}
