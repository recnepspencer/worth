use super::WorthQueryProgramApplicationRuntime;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use crate::domain_computation::primary_graph::WorthQueryApplicationDiscoveredOutputConnection;
use crate::equality_type_id::EqualityTypeId as TypeId;
use crate::facade::runtime::ExecutionAllocationPolicy;
use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationOutputGraphShape, ApplicationProgramDefinition,
    ApplicationProgramOutputsShape,
};

mod correction;
mod promotion;
mod recovery;
mod selected;
pub use recovery::{
    WorthQueryRecoveredProgramOutputSource, WorthQueryUnpublishedProgramOutputSource,
};

type RootConnectionRef<Schema, Root> =
    <Root as ApplicationOutputGraphShape<Schema>>::RootConnection;
type RootConnection<Schema, Root> =
    <RootConnectionRef<Schema, Root> as ApplicationConnectionShape<Schema>>::Binding;

type PreparedProgramSource = (
    crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
    std::sync::Arc<crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation>,
);
type ProgramSourceCommit = Result<
    (
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome,
        Option<PreparedProgramSource>,
        Option<WorthQueryUnpublishedProgramOutputSource>,
    ),
    crate::domain_computation::primary_graph::WorthQueryRequiredOutputSourcePreparationFailure,
>;

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Outputs: ApplicationProgramOutputsShape<Schema>,
{
    pub fn complete_program_output_source(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) {
        self.backing
            .runtime
            .complete_prepared_output_source(receipt);
    }

    pub fn recover_discovered_program_source<Root>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Result<
        (
            <RootConnection<Schema, Root> as WorthQueryApplicationDiscoveredOutputConnection<
                Schema,
            >>::Discovery,
            std::sync::Arc<
                crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
            >,
            crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
        ),
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial,
    >
    where
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
    {
        if !self.contains_output_root::<Root>() {
            return Err(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignDemand,
                "discovered output root is not installed for this program",
            ));
        }
        self.backing.runtime.recover_discovered_output_source::<
            <RootConnection<Schema, Root> as WorthQueryApplicationDiscoveredOutputConnection<Schema>>::Discovery,
        >(receipt, TypeId::of::<Root>())
    }

    /// Commits one output source under `presented`, the program its caller
    /// resolved as owning that source; the occurrence gate still refuses it
    /// unless it is the program this occurrence runs.
    fn compare_and_commit_output_source<Source>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        presented: Option<
            crate::domain_computation::primary_graph::program_occurrence::WorthQueryPresentedProgram<'_>,
        >,
        program: crate::domain_computation::primary_graph::WorthQueryApplicationEffectProgram<
            Schema,
            Source::Operation,
            Source::Input,
            <Source::ScopeBinding as worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        idempotency: crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
        causality: crate::domain_computation::application_aftermath::ApplicationCommitCausality,
        payload: recovery::ProgramOutputSourcePayload,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> ProgramSourceCommit
    where
        Source: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
        Source::Input: Clone + Send + Sync + 'static,
    {
        let Some(presented) = presented else {
            return Ok((
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Denied(
                    crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::application_program_required(),
                ),
                None,
                None,
            ));
        };
        let source_preparation = self
            .backing
            .runtime
            .output_demands
            .begin_source_preparation(program.product_branch().occurrence());
        let branch = program.product_branch();
        let selected_program = presented.rendering().clone();
        let correction = match &causality {
            crate::domain_computation::application_aftermath::ApplicationCommitCausality::RecordedCorrection(source) => Some(recovery::UnpublishedCorrectionCommand::new(source.intent(), &idempotency)),
            _ => None,
        };
        let idempotency = correction
            .as_ref()
            .map_or(idempotency, |command| command.bind(idempotency));
        let pending = WorthQueryUnpublishedProgramOutputSource {
            correction,
            runtime_authority: self.backing.runtime.runtime.authority_identity().as_u64(),
            program: selected_program,
            binding: TypeId::of::<Source>(),
            idempotency,
            branch,
            payload,
            preparation: source_preparation,
        };
        match self.backing.runtime
            .compare_and_commit_application_for_required_output_source(execution, phase,
                &presented,
                program,
                idempotency,
                causality,
                allocation_policy,
            ) {
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Committed(
                mut receipt,
            ) => {
                let descriptive = receipt.clone();
                let carrier = WorthQueryRecoveredProgramOutputSource::from_committed(&mut receipt);
                let prepared = match self.promote_program_output_source(
                    access, pending, carrier, receipt,
                ) {
                    Ok(prepared) => prepared,
                    Err((denial, source, carrier)) => {
                        return Err(crate::domain_computation::primary_graph::WorthQueryRequiredOutputSourcePreparationFailure {
                            receipt: descriptive, denial, source, carrier,
                        });
                    }
                };
                Ok((
                    crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Committed(
                        descriptive,
                    ),
                    Some(prepared),
                    None,
                ))
            }
            outcome @ (crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::ProductUnpublished(_)
                | crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Indeterminate(_)) => {
                Ok((outcome, None, Some(pending)))
            }
            outcome @ crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::CleanupRequired(_)
                if matches!(&outcome,
                    crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::CleanupRequired(required)
                    if matches!(required.outcome(),
                        crate::domain_computation::primary_graph::WorthQueryApplicationUncommitted::ProductUnpublished(_)
                            | crate::domain_computation::primary_graph::WorthQueryApplicationUncommitted::Indeterminate(_))) => {
                // Cleanup and the pending source are independent original owners.
                Ok((outcome, None, Some(pending)))
            }
            outcome => Ok((outcome, None, None)),
        }
    }
}
