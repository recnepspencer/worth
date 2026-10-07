use super::WorthQueryProgramApplicationRuntime;
use crate::domain_computation::primary_graph::WorthQueryApplicationDiscoveredOutputConnection;
use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationOutputGraphShape, ApplicationProgramDefinition,
    ApplicationProgramOutputsShape,
};

mod selected;

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
        self.runtime.complete_prepared_output_source(receipt);
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
        self.runtime.recover_discovered_output_source::<
            <RootConnection<Schema, Root> as WorthQueryApplicationDiscoveredOutputConnection<Schema>>::Discovery,
        >(receipt, std::any::TypeId::of::<Root>())
    }

    /// Commits one output source under `presented`, the program its caller
    /// resolved as owning that source; the occurrence gate still refuses it
    /// unless it is the program this occurrence runs.
    fn compare_and_commit_output_source<Source>(
        &self,
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
        root_kind: crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind,
        discovery: Option<std::sync::Arc<dyn std::any::Any + Send + Sync>>,
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
            ));
        };
        let source_preparation = self
            .runtime
            .output_demands
            .begin_source_preparation(program.product_branch().occurrence());
        match self
            .runtime
            .compare_and_commit_application_for_required_output_source(
                &presented,
                program,
                idempotency,
            ) {
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Committed(
                mut receipt,
            ) => {
                let descriptive = receipt.clone();
                let Some(change) = receipt.take_performed_relational_product_change() else {
                    return Err(crate::domain_computation::primary_graph::WorthQueryRequiredOutputSourcePreparationFailure {
                        receipt: descriptive,
                        denial: crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(
                            crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                            "committed required-output source has no performed-change carrier",
                        ),
                    });
                };
                let prepared = match self
                    .runtime
                    .retain_required_output_source(receipt, change, &source_preparation, root_kind, discovery)
                {
                    Ok(prepared) => prepared,
                    Err(denial) => {
                        return Err(crate::domain_computation::primary_graph::WorthQueryRequiredOutputSourcePreparationFailure {
                            receipt: descriptive,
                            denial,
                        })
                    }
                };
                Ok((
                    crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Committed(
                        descriptive,
                    ),
                    Some(prepared),
                ))
            }
            outcome @ (crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
            | crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::ProductStale(_)
            | crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::ProductUnpublished(_)
            | crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Stale(_)
            | crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Cancelled
            | crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::TimedOut
            | crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Denied(_)
            | crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Aborted
            | crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Deferred(_)
            | crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::NoEffect(_)
            | crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::SettlementDeferred(_)
            | crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Indeterminate(_)) => Ok((outcome, None)),
        }
    }
}
