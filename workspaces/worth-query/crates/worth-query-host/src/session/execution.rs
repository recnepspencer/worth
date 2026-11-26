//! One-shot mutation execution methods on [`WorthQueryHostSession`], charging
//! seed reads and occupied-key probes cumulatively against one metered request
//! and never retrying a recovered commit of the current attempt.
use worth_execution::ExecutionAllocationPolicy;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationIntent, ApplicationMutationScopeBinding,
        ApplicationMutationScopeResolution, NoApplicationMutationSource,
    },
    application_program::ApplicationProgramDefinition,
    application_schema::ApplicationSchema,
};
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationProgramMutationPreparation,
    WorthQueryApplicationRequestMutationDenial,
};

use super::auto_key::{durable_branch_seed_in_advancement, MAX_AUTO_KEY_PROBES};
use super::{
    WorthQueryAutoIdempotencyKey, WorthQueryHostSession, WorthQuerySessionCommit,
    WorthQuerySessionRunDenial,
};

fn map_mutation_outcome<D, R>(
    outcome: WorthQueryApplicationMutationOutcome<D, R>,
) -> Result<WorthQuerySessionCommit<R>, WorthQuerySessionRunDenial<D>> {
    match outcome {
        WorthQueryApplicationMutationOutcome::Committed { receipt, result } => {
            Ok(WorthQuerySessionCommit { receipt, result })
        }
        WorthQueryApplicationMutationOutcome::DomainDenied(denial) => {
            Err(WorthQuerySessionRunDenial::DomainDenied(denial))
        }
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt) => {
            Err(WorthQuerySessionRunDenial::AlreadyCommitted(receipt))
        }
        WorthQueryApplicationMutationOutcome::RecoveredCommitted(receipt) => {
            Err(WorthQuerySessionRunDenial::RecoveredCommitted(receipt))
        }
        WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift => {
            Err(WorthQuerySessionRunDenial::IdempotencyIntentDrift)
        }
        WorthQueryApplicationMutationOutcome::Cancelled => {
            Err(WorthQuerySessionRunDenial::Cancelled)
        }
        WorthQueryApplicationMutationOutcome::DeadlineExceeded => {
            Err(WorthQuerySessionRunDenial::DeadlineExceeded)
        }
        WorthQueryApplicationMutationOutcome::Commit(uncommitted) => {
            Err(WorthQuerySessionRunDenial::Uncommitted(uncommitted))
        }
    }
}

/// Only pre-submission settlement (from `prepare` before `commit` runs) proves
/// an existing prior entry owns the key; `RecoveredCommitted` after `commit` is
/// NEVER treated as an occupied collision to retry.
fn is_pre_submission_occupied<D, R>(outcome: &WorthQueryApplicationMutationOutcome<D, R>) -> bool {
    matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(_)
            | WorthQueryApplicationMutationOutcome::RecoveredCommitted(_)
            | WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
    )
}

impl<Schema, Program> WorthQueryHostSession<'_, Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    /// Runs a sourceless program mutation with a deterministic, restart-safe
    /// auto idempotency key derived from canonical durable branch state,
    /// charging cumulative work across seed read and probes to one request.
    pub fn run<Intent>(
        &self,
        intent: Intent,
    ) -> Result<
        WorthQuerySessionCommit<<Intent::Binding as ApplicationMutationBinding<Schema>>::Result>,
        WorthQuerySessionRunDenial<<Intent::Binding as ApplicationMutationBinding<Schema>>::Denial>,
    >
    where
        Intent: ApplicationMutationIntent<Schema> + Clone + Send + Sync,
        Intent::Binding:
            ApplicationMutationBinding<Schema, SourceExpectation = NoApplicationMutationSource>,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey:
            WorthQueryAutoIdempotencyKey,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                Schema,
                <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
            >,
    {
        let scope = self.fresh_scope();
        let lease = self.open_lease(&scope).map_err(|cause| {
            WorthQuerySessionRunDenial::Request(
                WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause),
            )
        })?;
        let allocation = ExecutionAllocationPolicy::Execution(&lease);
        let graph = self.home.primary_graph();
        graph
            .with_funded_application_completion(&scope, allocation, |phase| {
                let seed =
                    durable_branch_seed_in_advancement(graph, self.settings.branch, &phase).map_err(
                        |cause| {
                            WorthQuerySessionRunDenial::Request(
                                WorthQueryApplicationRequestMutationDenial::ProductSelection(cause),
                            )
                        },
                    )?;
                let principal = self.call_principal();
                let request = self.request_as(&principal, &scope);
                for probe in 0..MAX_AUTO_KEY_PROBES {
                    let key =
                        <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey::derive_auto_key(
                            seed, probe,
                        );
                    let mut staged = request
                        .advanced()
                        .mutate(intent.clone())
                        .without_source()
                        .idempotency(&key);
                    let prepared = staged
                        .prepare_in_program_report_in_advancement(
                            &phase,
                            self.home.runtime(),
                            allocation,
                        )
                        .into_outcome()
                        .map_err(WorthQuerySessionRunDenial::Request)?;
                    match prepared {
                        WorthQueryApplicationProgramMutationPreparation::Settled(outcome)
                            if is_pre_submission_occupied(&outcome) =>
                        {
                            continue;
                        }
                        WorthQueryApplicationProgramMutationPreparation::Settled(outcome) => {
                            return map_mutation_outcome(outcome);
                        }
                        WorthQueryApplicationProgramMutationPreparation::Prepared(candidate) => {
                            let committed = candidate
                                .commit_report_in_advancement(&phase, allocation)
                                .into_outcome();
                            if matches!(
                                committed,
                                WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
                            ) {
                                continue;
                            }
                            return map_mutation_outcome(committed);
                        }
                    }
                }
                Err(WorthQuerySessionRunDenial::AutoIdempotencyKeysExhausted)
            })
            .map_err(|cause| {
                WorthQuerySessionRunDenial::Request(
                    WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause),
                )
            })?
    }

    /// Runs a sourceless program mutation with an explicit caller idempotency key.
    pub fn run_with_key<Intent>(
        &self,
        intent: Intent,
        idempotency_key: &<Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey,
    ) -> Result<
        WorthQuerySessionCommit<<Intent::Binding as ApplicationMutationBinding<Schema>>::Result>,
        WorthQuerySessionRunDenial<<Intent::Binding as ApplicationMutationBinding<Schema>>::Denial>,
    >
    where
        Intent: ApplicationMutationIntent<Schema> + Clone + Send + Sync,
        Intent::Binding:
            ApplicationMutationBinding<Schema, SourceExpectation = NoApplicationMutationSource>,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                Schema,
                <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
            >,
    {
        let scope = self.fresh_scope();
        let lease = self.open_lease(&scope).map_err(|cause| {
            WorthQuerySessionRunDenial::Request(
                WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause),
            )
        })?;
        let principal = self.call_principal();
        let request = self.request_as(&principal, &scope);
        let outcome = request
            .advanced()
            .mutate(intent)
            .without_source()
            .idempotency(idempotency_key)
            .execute_in_program(
                self.home.runtime(),
                ExecutionAllocationPolicy::Execution(&lease),
            )
            .map_err(WorthQuerySessionRunDenial::Request)?;
        map_mutation_outcome(outcome)
    }

    /// Runs a capability-governed sourceless program mutation with a
    /// deterministic, restart-safe auto idempotency key under one metered request.
    pub fn run_governed<Intent>(
        &self,
        intent: Intent,
    ) -> Result<
        WorthQuerySessionCommit<<Intent::Binding as ApplicationMutationBinding<Schema>>::Result>,
        WorthQuerySessionRunDenial<<Intent::Binding as ApplicationMutationBinding<Schema>>::Denial>,
    >
    where
        Intent: ApplicationMutationIntent<Schema> + Clone + Send + Sync,
        Intent::Binding: ApplicationCapabilityMutationBinding<
            Schema,
            SourceExpectation = NoApplicationMutationSource,
        >,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone
            + Send
            + Sync
            + ApplicationCapabilityRequest<
                Schema,
                <Intent::Binding as ApplicationCapabilityMutationBinding<Schema>>::Capability,
                Scope = <<Intent::Binding as ApplicationMutationBinding<
                    Schema,
                >>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
            >,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey:
            WorthQueryAutoIdempotencyKey,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                Schema,
                <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
            >,
{
        let scope = self.fresh_scope();
        let lease = self.open_lease(&scope).map_err(|cause| {
            WorthQuerySessionRunDenial::Request(
                WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause),
            )
        })?;
        let allocation = ExecutionAllocationPolicy::Execution(&lease);
        let graph = self.home.primary_graph();
        graph
            .with_funded_application_completion(&scope, allocation, |phase| {
                let seed =
                    durable_branch_seed_in_advancement(graph, self.settings.branch, &phase).map_err(
                        |cause| {
                            WorthQuerySessionRunDenial::Request(
                                WorthQueryApplicationRequestMutationDenial::ProductSelection(cause),
                            )
                        },
                    )?;
                let principal = self.call_principal();
                let request = self.request_as(&principal, &scope);
                for probe in 0..MAX_AUTO_KEY_PROBES {
                    let key =
                        <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey::derive_auto_key(
                            seed, probe,
                        );
                    let mut staged = request
                        .advanced()
                        .mutate(intent.clone())
                        .without_source()
                        .idempotency(&key);
                    let prepared = staged
                        .prepare_capability_in_program_in_advancement(
                            &phase,
                            self.home.runtime(),
                            allocation,
                        )
                        .map_err(WorthQuerySessionRunDenial::Request)?;
                    match prepared {
                        WorthQueryApplicationProgramMutationPreparation::Settled(outcome)
                            if is_pre_submission_occupied(&outcome) =>
                        {
                            continue;
                        }
                        WorthQueryApplicationProgramMutationPreparation::Settled(outcome) => {
                            return map_mutation_outcome(outcome);
                        }
                        WorthQueryApplicationProgramMutationPreparation::Prepared(candidate) => {
                            let committed = candidate
                                .commit_report_in_advancement(&phase, allocation)
                                .into_outcome();
                            if matches!(
                                committed,
                                WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
                            ) {
                                continue;
                            }
                            return map_mutation_outcome(committed);
                        }
                    }
                }
                Err(WorthQuerySessionRunDenial::AutoIdempotencyKeysExhausted)
            })
            .map_err(|cause| {
                WorthQuerySessionRunDenial::Request(
                    WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause),
                )
            })?
    }

    /// Runs a capability-governed sourceless program mutation with an explicit key.
    pub fn run_governed_with_key<Intent>(
        &self,
        intent: Intent,
        idempotency_key: &<Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey,
    ) -> Result<
        WorthQuerySessionCommit<<Intent::Binding as ApplicationMutationBinding<Schema>>::Result>,
        WorthQuerySessionRunDenial<<Intent::Binding as ApplicationMutationBinding<Schema>>::Denial>,
    >
    where
        Intent: ApplicationMutationIntent<Schema> + Clone + Send + Sync,
        Intent::Binding: ApplicationCapabilityMutationBinding<
            Schema,
            SourceExpectation = NoApplicationMutationSource,
        >,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone
            + Send
            + Sync
            + ApplicationCapabilityRequest<
                Schema,
                <Intent::Binding as ApplicationCapabilityMutationBinding<Schema>>::Capability,
                Scope = <<Intent::Binding as ApplicationMutationBinding<
                    Schema,
                >>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
            >,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                Schema,
                <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
            >,
{
        let scope = self.fresh_scope();
        let lease = self.open_lease(&scope).map_err(|cause| {
            WorthQuerySessionRunDenial::Request(
                WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause),
            )
        })?;
        let principal = self.call_principal();
        let request = self.request_as(&principal, &scope);
        let outcome = request
            .advanced()
            .mutate(intent)
            .without_source()
            .idempotency(idempotency_key)
            .execute_capability_in_program(
                self.home.runtime(),
                ExecutionAllocationPolicy::Execution(&lease),
            )
            .map_err(WorthQuerySessionRunDenial::Request)?;
        map_mutation_outcome(outcome)
    }
}
