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
mod admission;
mod advance;
mod promotion;
mod retention;

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
        WorthQueryDiscoveredProgramOutputHandle<Schema, Program, Root>,
        WorthQueryRequiredOutputPreparationDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
        Root::Dependents: ProgramOutputContinuationFactory<Schema, Program, RootDemand<Schema, Root>,
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
        application
            .runtime()
            .with_application_advancement(self.scope, |phase| {
                self.recover_discovered_required_outputs_in_advancement(
                    &phase,
                    application,
                    receipt,
                    controls,
                )
            })
            .map_err(WorthQueryRequiredOutputPreparationDenial::advancement)?
    }

    pub(in crate::application_entry) fn recover_discovered_required_outputs_in_advancement<Program, Root>(
        &self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        receipt: &WorthQueryApplicationCommitReceipt,
        controls: WorthQueryOutputDemandControls,
    ) -> Result<
        WorthQueryDiscoveredProgramOutputHandle<Schema, Program, Root>,
        WorthQueryRequiredOutputPreparationDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
        Root::Dependents: ProgramOutputContinuationFactory<Schema, Program, RootDemand<Schema, Root>,
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
        application
            .validate_discovered_program_source::<Root>(
                phase,
                &worth_query_execution::publication_boundary::program_publication_access(),
                &prepared,
                receipt,
                &retained,
                self.principal,
                self.scope,
                self.branch,
            )
            .map_err(WorthQueryRequiredOutputPreparationDenial::DemandExecution)?;
        Ok(WorthQueryDiscoveredProgramOutputHandle::new(
            receipt.clone(),
            discovery,
            prepared,
            WorthQueryApplicationReadObservation::new(retained),
            controls,
            super::DiscoveredRootStartKind::Recovery,
        ))
    }
    pub(in crate::application_entry) fn recover_admitted_discovered_outputs_in_advancement<Program, Root, Operation, Input, Scope>(
        &self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        receipt: &WorthQueryApplicationCommitReceipt,
        controls: WorthQueryOutputDemandControls,
        admission: &worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        WorthQueryDiscoveredProgramOutputHandle<Schema, Program, Root>,
        AdmittedDiscoveredRecoveryDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
        Root::Dependents: ProgramOutputContinuationFactory<Schema, Program, RootDemand<Schema, Root>,
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
        admission
            .validate_current_authority()
            .map_err(AdmittedDiscoveredRecoveryDenial::Authorization)?;
        self.recover_discovered_required_outputs_in_advancement::<Program, Root>(
            phase,
            application,
            receipt,
            controls,
        )
        .map_err(AdmittedDiscoveredRecoveryDenial::Outputs)
    }
}

/// Admission lifecycle refusal and registered-output posture remain distinct.
pub(in crate::application_entry) enum AdmittedDiscoveredRecoveryDenial {
    Authorization(
        worth_query_execution::facade::primary_graph::WorthQueryOperationAuthorizationDenial,
    ),
    Outputs(WorthQueryRequiredOutputPreparationDenial),
}
