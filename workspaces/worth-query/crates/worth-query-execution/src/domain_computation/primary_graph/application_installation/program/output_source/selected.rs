//! Output sources committed under the program a branch selected.
//!
//! The initial program owns sources only on occurrences that still run it.
//! After a branch adopts a rostered successor, its sources commit under that
//! successor, presented through the owner resolved from the branch's own
//! selection. The occurrence gate still decides: a successor the occurrence
//! no longer runs is refused before any effect.

use std::any::TypeId;

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding,
};
use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition, ApplicationProgramOutputsShape,
};

use super::super::{WorthQueryProgramApplicationRuntime, WorthQuerySelectedProgramOwner};
use super::{ProgramSourceCommit, RootConnection};
use crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind;
use crate::domain_computation::primary_graph::program_occurrence::WorthQueryPresentedProgram;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationDiscoveredOutputConnection, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationRequiredOutputConnection,
    WorthQueryApplicationRequiredOutputSource,
};
use crate::publication_boundary::WorthQueryProgramPublicationAccess;

type SourceProgram<Schema, Source> = WorthQueryApplicationEffectProgram<
    Schema,
    <Source as ApplicationMutationBinding<Schema>>::Operation,
    <Source as ApplicationMutationBinding<Schema>>::Input,
    <<Source as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<
        Schema,
    >>::Scope,
>;

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Outputs: ApplicationProgramOutputsShape<Schema>,
{
    /// Commits a required-output source under the branch-selected program.
    pub fn compare_and_commit_selected_required_output_source<Root, Source>(
        &self,
        _: &WorthQueryProgramPublicationAccess,
        owner: &WorthQuerySelectedProgramOwner<'_, Schema>,
        program: SourceProgram<Schema, Source>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> ProgramSourceCommit
    where
        Source: ApplicationMutationBinding<Schema>,
        Root: ApplicationOutputGraphShape<Schema>,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
        Source: WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
        Source::Input: Clone + Send + Sync + 'static,
    {
        let root = TypeId::of::<Root>();
        let presented = match self.selected_source_owner::<Source>(owner, root) {
            Ok(presented) => presented,
            Err(denial) => return Ok((WorthQueryApplicationCommitOutcome::Denied(denial), None)),
        };
        self.compare_and_commit_output_source::<Source>(
            Some(presented),
            program,
            idempotency,
            PreparedOutputRootKind::Required(root),
            None,
        )
    }

    /// Commits a discovered-output source under the branch-selected program.
    pub fn compare_and_commit_selected_discovered_output_source<Root, Source>(
        &self,
        _: &WorthQueryProgramPublicationAccess,
        owner: &WorthQuerySelectedProgramOwner<'_, Schema>,
        program: SourceProgram<Schema, Source>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        discovery: <RootConnection<Schema, Root> as WorthQueryApplicationDiscoveredOutputConnection<Schema>>::Discovery,
    ) -> ProgramSourceCommit
    where
        Source: ApplicationMutationBinding<Schema>,
        Root: ApplicationOutputGraphShape<Schema>,
        RootConnection<Schema, Root>:
            WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Source>,
        Source::Input: Clone + Send + Sync + 'static,
    {
        let root = TypeId::of::<Root>();
        let presented = match self.selected_source_owner::<Source>(owner, root) {
            Ok(presented) => presented,
            Err(denial) => return Ok((WorthQueryApplicationCommitOutcome::Denied(denial), None)),
        };
        self.compare_and_commit_output_source::<Source>(
            Some(presented),
            program,
            idempotency,
            PreparedOutputRootKind::Discovered(root),
            Some(std::sync::Arc::new(discovery)),
        )
    }

    /// Presents `owner`'s program for `Source` under `root`, only when the
    /// owner was resolved on this host and both the initial program's output
    /// shape and the selected program declare that source and root. A selected
    /// revision whose support has since retired is refused as not active.
    fn selected_source_owner<Source: 'static>(
        &self,
        owner: &WorthQuerySelectedProgramOwner<'_, Schema>,
        root: TypeId,
    ) -> Result<WorthQueryPresentedProgram<'_>, WorthQueryApplicationCommitDenial> {
        use super::super::WorthQueryProgramOwner;

        let source = TypeId::of::<Source>();
        if !std::ptr::eq(owner.owned_runtime(), &self.runtime)
            || !self.root_graph_types.contains(&root)
            || !owner.owns_output_root(root)
            || !owner.owns_output_source(source)
        {
            return Err(WorthQueryApplicationCommitDenial::application_program_required());
        }
        self.runtime
            .installed_program_support()
            .ok_or_else(WorthQueryApplicationCommitDenial::application_program_required)?
            .present_for_commit(owner.owned_revision())
    }
}
