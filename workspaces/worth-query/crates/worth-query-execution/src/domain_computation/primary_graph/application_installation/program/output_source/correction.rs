//! Recorded correction uses the ordinary selected required-output source owner.
use super::selected::SourceProgram;
use super::*;
use crate::domain_computation::application_aftermath::ApplicationCommitCausality;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource,
    WorthQueryPreparedCorrectionSource,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities,
};

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    /// Publishes the sealed inverse and reserves its ordinary required-output custody.
    pub fn commit_required_output_correction<Root, Source>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        owner: &super::super::WorthQuerySelectedProgramOwner<'_, Schema>,
        program: SourceProgram<Schema, Source>,
        identities: &ApplicationMutationIdentities<'_, Schema, Source>,
        source: std::sync::Arc<WorthQueryPreparedCorrectionSource>,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> ProgramSourceCommit
    where
        Root: ApplicationOutputGraphShape<Schema>,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
        Source: ApplicationMutationBinding<Schema>
            + WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
        Source::Input: Clone + Send + Sync + 'static,
    {
        if !source.is_action_binding::<Source>()
            || !source.matches_input(identities.input_identity())
            || !source.requires_root(Some(TypeId::of::<Root>()))
        {
            return Ok((
                WorthQueryApplicationCommitOutcome::Denied(
                    WorthQueryApplicationCommitDenial::correction_context_mismatch(),
                ),
                None,
                None,
            ));
        }
        let idempotency = crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding::for_mutation_identities(identities);
        self.commit_selected_required_source::<Root, Source>(
            phase,
            access,
            owner,
            program,
            idempotency,
            ApplicationCommitCausality::RecordedCorrection(source),
            allocation_policy,
        )
    }
}
