//! A program-gated candidate retained between preparation and publication.

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_execution::facade::application_installation::{
    WorthQueryProgramApplicationRuntime, WorthQueryProgramOwner, WorthQuerySelectedProgramOwner,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    commit_binding::WorthQueryMutationCommitBinding,
    preparation::{CandidatePreparation, PreparedCandidate},
    selected_program::map_selected_program_owner_denial,
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationMutationRequestWithIdempotency,
};
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;

/// Preparation either retains an unpublished candidate or settles without a new result.
pub enum WorthQueryApplicationProgramMutationPreparation<
    'request,
    'application,
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
> {
    Prepared(WorthQueryPreparedProgramMutation<'request, 'application, Schema, Binding, Program>),
    Settled(WorthQueryApplicationMutationOutcome<Binding::Denial, Binding::Result>),
}

/// An installed handler candidate and its exact platform read set, awaiting commit.
///
/// Preparation has no effects and registers no idempotency outcome. Dropping this
/// handle discards the candidate. The request remains borrowed so the input and
/// key cannot change; no branch commit lane is held. The result is private until
/// `commit` compares the captured basis, current authority and selected program.
/// Ordinary candidates may publish after unrelated edits when their sealed facts,
/// current authority, selected program, Signal and Correspondence remain equal.
/// Definition and exact-parent candidates retain their original basis. A further
/// head change after readmission still fails the native publication comparison.
///
/// An unpublished result cannot be extracted:
///
/// ```compile_fail,E0616
/// use worth_query_publication::facade::application_entry::WorthQueryPreparedProgramMutation;
/// use worth_query_declaration::facade::{application_operation::ApplicationMutationBinding,
///     application_program::ApplicationProgramDefinition};
/// use worth_query_installation::facade::ApplicationSchema;
/// fn extract<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>,
///     Program: ApplicationProgramDefinition<Schema>>(
///     candidate: WorthQueryPreparedProgramMutation<'_, '_, Schema, Binding, Program>,
/// ) { let _result = candidate.candidate.result; }
/// ```
///
/// The supported transition publishes only through comparison:
///
/// ```
/// use worth_query_publication::facade::application_entry::{
///     WorthQueryApplicationMutationOutcome, WorthQueryPreparedProgramMutation};
/// use worth_query_declaration::facade::{application_operation::ApplicationMutationBinding,
///     application_program::ApplicationProgramDefinition};
/// use worth_query_installation::facade::ApplicationSchema;
/// fn publish<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>,
///     Program: ApplicationProgramDefinition<Schema>>(
///     candidate: WorthQueryPreparedProgramMutation<'_, '_, Schema, Binding, Program>,
/// ) -> WorthQueryApplicationMutationOutcome<Binding::Denial, Binding::Result>
/// where Binding::Input: Clone + Send + Sync { candidate.commit() }
/// ```
pub struct WorthQueryPreparedProgramMutation<
    'request,
    'application,
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
> {
    candidate: PreparedCandidate<'request, Schema, Binding>,
    application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    selected_owner: Option<WorthQuerySelectedProgramOwner<'application, Schema>>,
}

impl<Schema, Binding, Program> WorthQueryPreparedProgramMutation<'_, '_, Schema, Binding, Program>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Binding::Input: Clone + Send + Sync,
    Program: ApplicationProgramDefinition<Schema>,
{
    /// Consumes this candidate through the authoritative program commit path.
    /// Denial, cancellation and duplicate replay never release the candidate result.
    pub fn commit(self) -> WorthQueryApplicationMutationOutcome<Binding::Denial, Binding::Result> {
        let PreparedCandidate {
            program,
            result,
            identities,
            extension,
        } = self.candidate;
        let binding = WorthQueryMutationCommitBinding::new(&identities, extension);
        let outcome = match self.selected_owner {
            Some(owner) => owner.compare_and_commit_program_action(
                program,
                binding.identities(),
                |idempotency| binding.extension().apply(idempotency),
            ),
            None => self.application.compare_and_commit_program_action(
                program,
                binding.identities(),
                |idempotency| binding.extension().apply(idempotency),
            ),
        };
        match outcome.landed() {
            Ok((receipt, false)) => {
                WorthQueryApplicationMutationOutcome::Committed { receipt, result }
            }
            Ok((receipt, true)) => WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt),
            Err(uncommitted) => WorthQueryApplicationMutationOutcome::Commit(uncommitted),
        }
    }
}

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
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema> + Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    /// Runs the installed decision and candidate phases, retaining the unpublished
    /// result outside the commit lane. Call `commit` once ready to publish.
    pub fn prepare_in_program<'request, Program>(
        &'request mut self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryApplicationProgramMutationPreparation<
            'request,
            'application,
            Schema,
            Intent::Binding,
            Program,
        >,
        WorthQueryApplicationRequestMutationDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        if !std::ptr::eq(application.runtime(), self.request.application) {
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch);
        }
        let selected = self
            .request
            .application
            .on_branch(self.request.branch)
            .select()
            .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)?;
        let owner = application
            .selected_program_owner(&selected)
            .map_err(map_selected_program_owner_denial)?;
        let selected_owns_action = owner.contains_action::<Intent::Binding>();
        if !selected_owns_action && !application.contains_action::<Intent::Binding>() {
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired);
        }
        match self.prepare_candidate(move |request, identities, staged| {
            super::authorization::prepare_selected(request, identities, staged, &selected)
        })? {
            CandidatePreparation::Prepared(candidate) => {
                Ok(WorthQueryApplicationProgramMutationPreparation::Prepared(
                    WorthQueryPreparedProgramMutation {
                        candidate,
                        application,
                        selected_owner: selected_owns_action.then_some(owner),
                    },
                ))
            }
            CandidatePreparation::Settled(outcome) => Ok(
                WorthQueryApplicationProgramMutationPreparation::Settled(outcome),
            ),
        }
    }
}
