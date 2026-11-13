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
mod envelope;
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
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        source: impl Into<crate::domain_computation::primary_graph::WorthQueryOutputSourceEvidence>,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial> {
        use crate::domain_computation::primary_graph::{
            WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind as Kind,
        };
        let receipt = source.into();
        let execution = phase
            .execution_request_for(&self.runtime().runtime.product_runtime)
            .map_err(|cause| {
                WorthQueryOutputDemandDenial::new(
                    Kind::ExecutionRequest(cause.into()),
                    "source completion",
                )
            })?;
        if let Some(owner) = receipt.source_commit().owner_identity().durable() {
            let commit = worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
                owner,
                receipt.source_commit().ordinal(),
            );
            self.runtime()
                .installed_program_support()
                .ok_or_else(|| {
                    WorthQueryOutputDemandDenial::new(
                        Kind::ForeignSource,
                        "source support unavailable",
                    )
                })?
                .lifecycle()
                .flush_output_discharges(&execution)
                .and_then(|()| {
                    self.runtime()
                        .installed_program_support()
                        .expect("same support")
                        .lifecycle()
                        .discharge_output_source(&execution, commit)
                })
                .map_err(|cause| {
                    WorthQueryOutputDemandDenial::new(
                        Kind::ObligationPublication(cause),
                        "source completion",
                    )
                })?;
        }
        self.backing
            .runtime()
            .complete_prepared_output_source(&receipt);
        Ok(())
    }

    pub fn recover_discovered_program_source<Root>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        selector: crate::domain_computation::primary_graph::WorthQueryOutputSourceReplay<'_>,
    ) -> Result<
        (
            crate::domain_computation::primary_graph::WorthQueryOutputSourceEvidence,
            crate::domain_computation::primary_graph::WorthQueryOutputDiscoveryRecipe<
                <RootConnection<Schema, Root> as WorthQueryApplicationDiscoveredOutputConnection<
                    Schema,
                >>::Discovery,
            >,
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
        let receipt = self.backing.runtime().resolve_output_source_replay(phase, selector,
            crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind::Discovered(TypeId::of::<Root>()),
            RootConnectionRef::<Schema, Root>::declaration().identity())?;
        let (discovery, observation, prepared) = self.backing.runtime().recover_discovered_output_source::<
            <RootConnection<Schema, Root> as WorthQueryApplicationDiscoveredOutputConnection<Schema>>::Discovery,
        >(&receipt, TypeId::of::<Root>())?;
        Ok((receipt, discovery, observation, prepared))
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
        root_identity: &'static str,
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
            .runtime()
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
        let discovery_recipe = match program.synchronous_outputs.iter().find_map(|root| root.source_recipe()) {
            Some(Ok(value)) => Some(value.clone()),
            None => None,
            Some(Err(_)) => return Ok((crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Denied(crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::required_output_preparation_missing(Source::IDENTITY)),None,None)),
        };
        let pending = WorthQueryUnpublishedProgramOutputSource {
            correction,
            runtime_authority: self.backing.runtime().runtime.authority_identity().as_u64(),
            revision: worth_foundational::facade::CanonicalDigestId::new(
                *presented.revision().as_bytes(),
            ),
            root_identity,
            discovery_recipe,
            program: selected_program,
            binding: TypeId::of::<Source>(),
            idempotency,
            branch,
            payload,
            preparation: source_preparation,
        };
        match self.backing.runtime()
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
                    phase, access, pending, carrier, receipt,
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
                        crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::ProductUnpublished(_)
                            | crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Indeterminate(_))) => {
                // Cleanup and the pending source are independent original owners.
                Ok((outcome, None, Some(pending)))
            }
            outcome => Ok((outcome, None, None)),
        }
    }
}
