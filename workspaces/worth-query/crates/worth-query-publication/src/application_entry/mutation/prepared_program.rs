//! A program-gated candidate retained between preparation and publication.
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase as AdvancementPhase;

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_execution::facade::application_installation::{
    WorthQueryProgramApplicationRuntime, WorthQueryProgramOwner, WorthQuerySelectedProgramOwner,
};
use worth_query_execution::facade::runtime::ExecutionAllocationPolicy;
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
/// use worth_query_execution::facade::runtime::ExecutionAllocationPolicy;
/// fn publish<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>,
///     Program: ApplicationProgramDefinition<Schema>>(
///     candidate: WorthQueryPreparedProgramMutation<'_, '_, Schema, Binding, Program>,
///     allocation_policy: ExecutionAllocationPolicy<'_, '_>,
/// ) -> WorthQueryApplicationMutationOutcome<Binding::Denial, Binding::Result>
/// where Binding::Input: Clone + Send + Sync { candidate.commit(allocation_policy) }
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
    pub fn commit(
        self,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationMutationOutcome<Binding::Denial, Binding::Result> {
        self.commit_report(allocation_policy).into_outcome()
    }

    /// Publishes once, retaining the original preparation's work even on duplicate commit.
    pub fn commit_report(
        self,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> super::WorthQueryApplicationMutationAttemptReport<
        WorthQueryApplicationMutationOutcome<Binding::Denial, Binding::Result>,
    > {
        let scope = self.candidate.program.request_scope().clone();
        let runtime = self.application.runtime();
        let work = self.candidate.decision_work;
        runtime
            .with_application_advancement(&scope, |phase| {
                self.commit_report_in_advancement(&phase, allocation_policy)
            })
            .unwrap_or_else(|cause| {
                super::WorthQueryApplicationMutationAttemptReport::new(
                    WorthQueryApplicationMutationOutcome::Commit(
                        cause
                            .into_commit_outcome()
                            .landed()
                            .expect_err("request admission cannot commit"),
                    ),
                    work,
                )
            })
    }

    pub(super) fn commit_report_in_advancement(
        self,
        phase: &AdvancementPhase<'_>,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> super::WorthQueryApplicationMutationAttemptReport<
        WorthQueryApplicationMutationOutcome<Binding::Denial, Binding::Result>,
    > {
        let PreparedCandidate {
            program,
            result,
            identities,
            extension,
            decision_work,
        } = self.candidate;
        let binding = WorthQueryMutationCommitBinding::new(&identities, extension);
        let outcome = match self.selected_owner {
            Some(owner) => owner.commit_program_action_in_advancement(
                phase,
                program,
                binding.identities(),
                |idempotency| binding.extension().apply(idempotency),
                allocation_policy,
            ),
            None => self.application.commit_program_action_in_advancement(
                phase,
                program,
                binding.identities(),
                |idempotency| binding.extension().apply(idempotency),
                allocation_policy,
            ),
        };
        let outcome = match outcome.landed() {
            Ok((receipt, false)) => {
                WorthQueryApplicationMutationOutcome::Committed { receipt, result }
            }
            Ok((receipt, true)) => WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt),
            Err(uncommitted) => WorthQueryApplicationMutationOutcome::Commit(uncommitted),
        };
        super::WorthQueryApplicationMutationAttemptReport::new(outcome, decision_work)
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
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
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
        self.prepare_in_program_report(application, allocation_policy)
            .into_outcome()
    }

    /// Reports decision work without releasing the candidate's unpublished result.
    pub fn prepare_in_program_report<'request, Program>(
        &'request mut self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> super::WorthQueryApplicationMutationAttemptReport<
        Result<
            WorthQueryApplicationProgramMutationPreparation<
                'request,
                'application,
                Schema,
                Intent::Binding,
                Program,
            >,
            WorthQueryApplicationRequestMutationDenial,
        >,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        let scope = self.request_scope().clone();
        let runtime = self.request.application;
        runtime
            .with_application_advancement(&scope, |phase| {
                let mut decision_work =
            worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork::NotStarted;
                let outcome = self.prepare_in_program_with_work(
                    &phase,
                    application,
                    &mut decision_work,
                    allocation_policy,
                );
                super::WorthQueryApplicationMutationAttemptReport::new(outcome, decision_work)
            })
            .unwrap_or_else(|cause| {
                super::WorthQueryApplicationMutationAttemptReport::new(
            Err(WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause)),
            worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork::NotStarted,
        )
            })
    }

    pub(super) fn prepare_in_program_with_work<'request, Program>(
        &'request mut self,
        phase: &AdvancementPhase<'_>,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        decision_work: &mut worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
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
        match self.prepare_candidate(
            phase,
            move |request, identities, staged| {
                super::authorization::prepare_selected(request, identities, staged, &selected)
            },
            decision_work,
            allocation_policy,
        )? {
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
