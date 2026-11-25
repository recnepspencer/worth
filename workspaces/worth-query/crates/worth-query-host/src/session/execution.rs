//! One-shot mutation and query execution methods on [`WorthQueryHostSession`].
use worth_execution::ExecutionAllocationPolicy;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationIntent, ApplicationMutationScopeBinding,
        ApplicationMutationScopeResolution, NoApplicationMutationSource,
    },
    application_program::ApplicationProgramDefinition,
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationSchema,
        ApplicationStructuredValueBinding,
    },
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApprovedElevation,
};
use worth_query_publication::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestQueryDenial,
    },
    domain_computation::WorthQueryPublishedApplicationResult,
};

use super::{
    QueryBinding, QueryMarker, QueryRow, QueryScope, WorthQueryAutoIdempotencyKey,
    WorthQueryHostSession, WorthQuerySessionCommit, WorthQuerySessionRunDenial,
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

impl<Schema, Program> WorthQueryHostSession<'_, Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    /// Runs a sourceless program mutation with an automatic restart-safe
    /// idempotency key.
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
        let key =
            <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey::next_restart_safe_key();
        self.run_with_key(intent, &key)
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
        let request = self.request(&scope);
        let outcome = request
            .advanced()
            .mutate(intent)
            .without_source()
            .idempotency(idempotency_key)
            .execute_in_program(
                self.home.runtime(),
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(WorthQuerySessionRunDenial::Request)?;
        map_mutation_outcome(outcome)
    }

    /// Runs a capability-governed sourceless program mutation with a fresh key.
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
        let key =
            <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey::next_restart_safe_key();
        self.run_governed_with_key(intent, &key)
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
        let request = self.request(&scope);
        let outcome = request
            .advanced()
            .mutate(intent)
            .without_source()
            .idempotency(idempotency_key)
            .execute_capability_in_program(
                self.home.runtime(),
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(WorthQuerySessionRunDenial::Request)?;
        map_mutation_outcome(outcome)
    }

    /// Executes a one-shot query and returns its typed projected rows directly.
    pub fn query<Intent>(
        &self,
        intent: Intent,
    ) -> Result<Vec<QueryRow<Schema, Intent>>, WorthQueryApplicationRequestQueryDenial>
    where
        Intent: ApplicationQueryIntent<Schema>,
        <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
        self.query_result(intent)
            .map(|published| published.into_rows())
    }

    /// Executes a one-shot query and retains the full published result (rows,
    /// publication receipt, and observed sources).
    pub fn query_result<Intent>(
        &self,
        intent: Intent,
    ) -> Result<
        WorthQueryPublishedApplicationResult<QueryMarker<Schema, Intent>, QueryRow<Schema, Intent>>,
        WorthQueryApplicationRequestQueryDenial,
    >
    where
        Intent: ApplicationQueryIntent<Schema>,
        <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
        let scope = self.fresh_scope();
        let request = self.request(&scope);
        let query_req = request.query(intent);
        let query_req = match self.query_result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        query_req.execute()
    }

    /// Executes a capability-governed one-shot query and returns its typed rows.
    pub fn query_governed<Intent, Capability, Operation, Input>(
        &self,
        intent: Intent,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<Vec<QueryRow<Schema, Intent>>, WorthQueryApplicationRequestQueryDenial>
    where
        Intent: ApplicationQueryIntent<Schema>,
        <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = QueryScope<Schema, Intent>>
            + 'static,
{
        let scope = self.fresh_scope();
        let request = self.request(&scope);
        let query_req = request.query(intent);
        let query_req = match self.query_result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        query_req
            .execute_governed(capability, operation, input)
            .map(|published| published.into_rows())
    }

    /// Executes an approved-elevation one-shot query and returns its typed rows.
    pub fn query_approved<Intent, Capability, Operation, Input>(
        &self,
        intent: Intent,
        approved: &WorthQueryApprovedElevation,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<Vec<QueryRow<Schema, Intent>>, WorthQueryApplicationRequestQueryDenial>
    where
        Intent: ApplicationQueryIntent<Schema>,
        <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = QueryScope<Schema, Intent>>
            + 'static,
{
        let scope = self.fresh_scope();
        let request = self.request(&scope);
        let query_req = request.query(intent);
        let query_req = match self.query_result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        query_req
            .execute_approved(approved, capability, operation, input)
            .map(|published| published.into_rows())
    }
}
