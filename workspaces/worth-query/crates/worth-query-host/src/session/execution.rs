//! One-shot mutation and query execution methods on [`WorthQueryHostSession`],
//! enforcing the session's configured [`ExecutionRequestPolicy`] on every lane.
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
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestMutationDenial,
};

use super::auto_key::{durable_branch_seed, DurableSeedDenial, MAX_AUTO_KEY_PROBES};
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

fn map_seed_denial<D>(denial: DurableSeedDenial) -> WorthQuerySessionRunDenial<D> {
    match denial {
        DurableSeedDenial::Advancement(cause) => WorthQuerySessionRunDenial::Request(
            WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause),
        ),
        DurableSeedDenial::Selection(cause) => WorthQuerySessionRunDenial::Request(
            WorthQueryApplicationRequestMutationDenial::ProductSelection(cause),
        ),
    }
}

fn is_occupied_key<D>(denial: &WorthQuerySessionRunDenial<D>) -> bool {
    matches!(
        denial,
        WorthQuerySessionRunDenial::AlreadyCommitted(_)
            | WorthQuerySessionRunDenial::RecoveredCommitted(_)
            | WorthQuerySessionRunDenial::IdempotencyIntentDrift
    )
}

impl<Schema, Program> WorthQueryHostSession<'_, Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    /// Runs a sourceless program mutation with a deterministic, restart-safe
    /// auto idempotency key derived from the branch's durable committed head.
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
        let seed = durable_branch_seed(self.home.primary_graph(), self.branch, self.policy, &scope)
            .map_err(map_seed_denial)?;
        let mut last_occupied = None;
        for probe in 0..MAX_AUTO_KEY_PROBES {
            let key =
                <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey::derive_auto_key(
                    seed, probe,
                );
            match self.run_with_key(intent.clone(), &key) {
                Ok(committed) => return Ok(committed),
                Err(denial) if is_occupied_key(&denial) => {
                    last_occupied = Some(denial);
                }
                Err(other) => return Err(other),
            }
        }
        Err(last_occupied.expect("probe loop ran at least once"))
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
        let request = self.request(&scope);
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
    /// deterministic, restart-safe auto idempotency key.
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
        let seed = durable_branch_seed(self.home.primary_graph(), self.branch, self.policy, &scope)
            .map_err(map_seed_denial)?;
        let mut last_occupied = None;
        for probe in 0..MAX_AUTO_KEY_PROBES {
            let key =
                <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey::derive_auto_key(
                    seed, probe,
                );
            match self.run_governed_with_key(intent.clone(), &key) {
                Ok(committed) => return Ok(committed),
                Err(denial) if is_occupied_key(&denial) => {
                    last_occupied = Some(denial);
                }
                Err(other) => return Err(other),
            }
        }
        Err(last_occupied.expect("probe loop ran at least once"))
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
        let request = self.request(&scope);
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
