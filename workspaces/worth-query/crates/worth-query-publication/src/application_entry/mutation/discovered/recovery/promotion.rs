//! Fresh keyed publication read followed by one-use discovered-source promotion.

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_program::{
    ApplicationDiscoveredOutputRoot, ApplicationOutputGraphShape, ApplicationProgramDefinition,
    ApplicationProgramOutputsShape,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationDiscoveredOutputConnection, WorthQueryApplicationIdempotencyResolution,
    WorthQueryApplicationProjection, WorthQueryManagedApplicationRecoveryDenial,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::unpublished::{DiscoveredRecoveryPhase, RecoveredCarrier};
use super::super::{
    DiscoveredRootStartKind, RootConnection, WorthQueryDiscoveredProgramOutputHandle,
    WorthQueryRecoveredDiscoveredOutputs, WorthQueryUnpublishedDiscoveredApplicationMutation,
};
use super::{
    DiscoveryBinding, DiscoveryQuery, DiscoveryValue, RootDemand, RootSource, RootSourceQuery,
    RootSourceValue,
};
use crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory;
use crate::application_entry::mutation::{
    WorthQueryApplicationMutationRequestWithIdempotency, WorthQueryApplicationRecoveryRequestDenial,
};
use crate::application_entry::{
    WorthQueryApplicationReadObservation, WorthQueryOutputDemandControls,
};

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    /// Reads the exact original key through fresh authorization and transfers
    /// the original performed source into its required-output handle once.
    ///
    /// The raw performed recovery and all earlier cleanup failures accompany
    /// the handle. Promotion does not discharge those independent obligations.
    /// Every refusal returns the complete move-only recovery owner.
    pub fn promote_recovered_discovered_outputs<Program, Root>(
        self,
        recovery: WorthQueryUnpublishedDiscoveredApplicationMutation<Schema, Intent, Program, Root>,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        controls: WorthQueryOutputDemandControls,
    ) -> Result<
        WorthQueryRecoveredDiscoveredOutputs<Schema, Program, Root>,
        (WorthQueryApplicationRecoveryRequestDenial, WorthQueryUnpublishedDiscoveredApplicationMutation<Schema, Intent, Program, Root>),
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::Outputs: ApplicationProgramOutputsShape<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema, Source=Intent::Binding>,
        Root::Dependents: ProgramOutputContinuationFactory<Schema, Program, RootDemand<Schema, Root>>,
        RootDemand<Schema, Root>: Clone,
        DiscoveryValue<Schema, Root>: WorthQueryApplicationProjection<Schema, DiscoveryQuery<Schema, Root>> + Clone,
        <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<Schema, <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
        RootSourceValue<Schema, Root>: WorthQueryApplicationProjection<Schema, RootSourceQuery<Schema, Root>> + Clone,
        <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Input:
            ApplicationQueryIntent<Schema, Binding=RootSource<Schema, Root>>,
        <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<Schema, <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
    {
        let runtime = self.request.application;
        let scope = self.request.scope.clone();
        let mut retained = Some(recovery);
        match runtime.with_application_advancement(&scope, |phase| {
            self.promote_recovered_discovered_outputs_in_advancement(
                &phase,
                retained
                    .take()
                    .expect("opened request owns recovery custody"),
                application,
                controls,
            )
        }) {
            Ok(outcome) => outcome,
            Err(cause) => Err((
                WorthQueryApplicationRecoveryRequestDenial::advancement(cause),
                retained.take().expect("refused request retains custody"),
            )),
        }
    }

    fn promote_recovered_discovered_outputs_in_advancement<Program, Root>(
        mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        recovery: WorthQueryUnpublishedDiscoveredApplicationMutation<Schema, Intent, Program, Root>,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        controls: WorthQueryOutputDemandControls,
    ) -> Result<
        WorthQueryRecoveredDiscoveredOutputs<Schema, Program, Root>,
        (WorthQueryApplicationRecoveryRequestDenial, WorthQueryUnpublishedDiscoveredApplicationMutation<Schema, Intent, Program, Root>),
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::Outputs: ApplicationProgramOutputsShape<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema, Source=Intent::Binding>,
        Root::Dependents: ProgramOutputContinuationFactory<Schema, Program, RootDemand<Schema, Root>>,
        RootDemand<Schema, Root>: Clone,
        DiscoveryValue<Schema, Root>: WorthQueryApplicationProjection<Schema, DiscoveryQuery<Schema, Root>> + Clone,
        <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<Schema, <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
        RootSourceValue<Schema, Root>: WorthQueryApplicationProjection<Schema, RootSourceQuery<Schema, Root>> + Clone,
        <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Input:
            ApplicationQueryIntent<Schema, Binding=RootSource<Schema, Root>>,
        <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<Schema, <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
    {
        let receipt = match self.published_discovered_receipt(phase, application, &recovery) {
            Ok(receipt) => receipt,
            Err(denial) => return Err((denial, recovery)),
        };
        let WorthQueryUnpublishedDiscoveredApplicationMutation {
            source,
            discovery,
            phase,
            prior_cleanup,
            initial_cause,
            program,
        } = recovery;
        let (carrier, outcome) = match phase {
            DiscoveredRecoveryPhase::Performed {
                carrier: RecoveredCarrier::Retained(carrier),
                outcome,
            } => (carrier, outcome),
            phase => {
                return Err((
                    binding_mismatch(),
                    WorthQueryUnpublishedDiscoveredApplicationMutation {
                        source,
                        discovery,
                        phase,
                        prior_cleanup,
                        initial_cause,
                        program,
                    },
                ))
            }
        };
        let (prepared, retained) = match application.promote_program_output_source(
            &worth_query_execution::publication_boundary::program_publication_access(),
            source,
            carrier,
            receipt.clone(),
        ) {
            Ok(prepared) => prepared,
            Err((denial, source, carrier)) => {
                return Err((
                    WorthQueryApplicationRecoveryRequestDenial::Recovery(
                        WorthQueryManagedApplicationRecoveryDenial::Demand(denial),
                    ),
                    WorthQueryUnpublishedDiscoveredApplicationMutation {
                        source,
                        discovery,
                        phase: DiscoveredRecoveryPhase::Performed {
                            carrier: RecoveredCarrier::Retained(carrier),
                            outcome,
                        },
                        prior_cleanup,
                        initial_cause,
                        program,
                    },
                ))
            }
        };
        // No further fallible preparation after registry transfer. The ordinary
        // output handle owns the prepared token and original observation now.
        Ok(WorthQueryRecoveredDiscoveredOutputs {
            outputs: WorthQueryDiscoveredProgramOutputHandle::new(
                receipt,
                discovery,
                prepared,
                WorthQueryApplicationReadObservation::new(retained),
                controls,
                DiscoveredRootStartKind::Recovery,
            ),
            initial_cause,
            performed: outcome,
            prior_cleanup,
        })
    }

    fn published_discovered_receipt<Program, Root>(
        &mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        recovery: &WorthQueryUnpublishedDiscoveredApplicationMutation<
            Schema,
            Intent,
            Program,
            Root,
        >,
    ) -> Result<
        worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
        WorthQueryApplicationRecoveryRequestDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::Outputs: ApplicationProgramOutputsShape<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>:
            WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
    {
        if !matches!(recovery.phase, DiscoveredRecoveryPhase::Performed { .. }) {
            return Err(binding_mismatch());
        }
        let prepared = self.authorize_discovered_source::<Program, Root>(
            phase,
            application,
            &recovery.source,
        )?;
        let read = self.read_recovery_idempotency(phase, &prepared)?;
        self.check_recovery_liveness()?;
        match read.into_resolution() {
            WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt) => Ok(receipt),
            WorthQueryApplicationIdempotencyResolution::Unseen
            | WorthQueryApplicationIdempotencyResolution::IntentDrift => Err(binding_mismatch()),
        }
    }
}

fn binding_mismatch() -> WorthQueryApplicationRecoveryRequestDenial {
    WorthQueryApplicationRecoveryRequestDenial::Recovery(
        WorthQueryManagedApplicationRecoveryDenial::BindingMismatch,
    )
}
