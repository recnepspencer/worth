//! Fresh keyed publication read followed by one-use required-source promotion.

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition, ApplicationProgramOutputsShape,
    ApplicationRequiredOutputRoot,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationIdempotencyResolution, WorthQueryApplicationRequiredOutputConnection,
    WorthQueryManagedApplicationRecoveryDenial,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::unpublished::{ProgramSourceRecoveryPhase, RecoveredCarrier};
use super::super::{
    RootConnection, WorthQueryRecoveredRequiredOutputs,
    WorthQueryUnpublishedRequiredApplicationMutation,
};
use crate::application_entry::mutation::{
    WorthQueryApplicationMutationRequestWithIdempotency, WorthQueryApplicationRecoveryRequestDenial,
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
    pub fn promote_recovered_required_outputs<Program, Root>(
        mut self,
        recovery: WorthQueryUnpublishedRequiredApplicationMutation<Schema, Intent, Program, Root>,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryRecoveredRequiredOutputs<Schema, Program, Root, Intent::Binding>,
        (
            WorthQueryApplicationRecoveryRequestDenial,
            WorthQueryUnpublishedRequiredApplicationMutation<Schema, Intent, Program, Root>,
        ),
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::Outputs: ApplicationProgramOutputsShape<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
        Intent::Binding:
            worth_query_execution::facade::primary_graph::WorthQueryApplicationRequiredOutputSource<
                Schema,
                RootConnection<Schema, Root>,
            >,
    {
        let receipt = match self.published_required_receipt(application, &recovery) {
            Ok(receipt) => receipt,
            Err(denial) => return Err((denial, recovery)),
        };
        let WorthQueryUnpublishedRequiredApplicationMutation {
            source,
            demand,
            phase,
            prior_cleanup,
            initial_cause,
            program,
        } = recovery;
        let (carrier, outcome) = match phase {
            ProgramSourceRecoveryPhase::Performed {
                carrier: RecoveredCarrier::Retained(carrier),
                outcome,
            } => (carrier, outcome),
            phase => {
                return Err((
                    binding_mismatch(),
                    WorthQueryUnpublishedRequiredApplicationMutation {
                        source,
                        demand,
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
                    WorthQueryUnpublishedRequiredApplicationMutation {
                        source,
                        demand,
                        phase: ProgramSourceRecoveryPhase::Performed {
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
        Ok(WorthQueryRecoveredRequiredOutputs {
            outputs: super::super::preparation::WorthQueryRequiredOutputPreparation {
                receipt,
                demand,
                prepared,
                retained_source: retained,
                source_bound: false,
                program: std::marker::PhantomData,
            },
            initial_cause,
            performed: outcome,
            prior_cleanup,
        })
    }

    fn published_required_receipt<Program, Root>(
        &mut self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        recovery: &WorthQueryUnpublishedRequiredApplicationMutation<Schema, Intent, Program, Root>,
    ) -> Result<
        worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
        WorthQueryApplicationRecoveryRequestDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::Outputs: ApplicationProgramOutputsShape<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
        Intent::Binding:
            worth_query_execution::facade::primary_graph::WorthQueryApplicationRequiredOutputSource<
                Schema,
                RootConnection<Schema, Root>,
            >,
    {
        if !matches!(recovery.phase, ProgramSourceRecoveryPhase::Performed { .. }) {
            return Err(binding_mismatch());
        }
        let prepared =
            self.authorize_required_source::<Program, Root>(application, &recovery.source)?;
        let read = self
            .request
            .application
            .resolve_admitted_application_idempotency(&prepared.admission, prepared.idempotency)
            .map_err(WorthQueryApplicationRecoveryRequestDenial::Idempotency)?;
        self.check_required_recovery_liveness()?;
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
