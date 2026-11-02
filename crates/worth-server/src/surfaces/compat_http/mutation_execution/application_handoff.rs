//! An installed application owns keyed mutation execution through Query.
mod admitted;
pub use admitted::WorthServerAdmittedQueryMutation;
mod selection;
pub use selection::WorthServerQueryMutationSelection;
use std::sync::Arc;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial,
    },
    application_installation::WorthQueryProgramApplicationRuntime,
    declaration::{
        application_operation::{
            ApplicationMutationBinding, ApplicationMutationIntent,
            ApplicationMutationScopeResolution, NoApplicationMutationSource,
        },
        application_program::ApplicationProgramDefinition,
    },
    domain::ApplicationSchema,
    runtime::ExecutionAllocationPolicy,
};

/// Encoding failures retain the actual application outcome and its receipt.
/// They never turn a landed commit into a retryable transport failure.
pub type WorthServerQueryMutationPayload = Result<serde_json::Value, String>;
pub type WorthServerQueryMutationAnswer = Result<
    Outcome<WorthServerQueryMutationPayload, WorthServerQueryMutationPayload>,
    WorthQueryApplicationRequestMutationDenial,
>;

/// Opaque installed adapter. Its callback resolves actual application inputs;
/// this owner always binds the admitted key and intent at the Query door.
/// Selection borrows this installed runtime, so no second runtime capture is needed.
#[derive(Clone)]
pub struct WorthServerQueryApplicationMutationBinding {
    execute: Arc<
        dyn Fn(&WorthServerAdmittedQueryMutation) -> WorthServerQueryMutationAnswer + Send + Sync,
    >,
}
impl std::fmt::Debug for WorthServerQueryApplicationMutationBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthServerQueryApplicationMutationBinding")
            .finish_non_exhaustive()
    }
}
impl WorthServerQueryApplicationMutationBinding {
    pub fn for_program<Schema, Program, Intent>(
        runtime: WorthQueryProgramApplicationRuntime<Schema, Program>,
        key_namespace: <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey,
        select: impl Fn(
                &WorthQueryProgramApplicationRuntime<Schema, Program>,
                &WorthServerAdmittedQueryMutation,
            ) -> Result<
                WorthServerQueryMutationSelection<Schema, Intent>,
                WorthQueryApplicationRequestMutationDenial,
            > + Send
            + Sync
            + 'static,
    ) -> Self
    where
        Schema: ApplicationSchema + Send + Sync + 'static,
        Program: ApplicationProgramDefinition<Schema> + Send + Sync + 'static,
        Intent: ApplicationMutationIntent<Schema> + Clone + Send + Sync + 'static,
        Intent::Binding:
            ApplicationMutationBinding<Schema, SourceExpectation = NoApplicationMutationSource>,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey:
            Send + Sync + 'static,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                Schema,
                <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
            >,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial: serde::Serialize,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Result: serde::Serialize,
    {
        Self {
            execute: Arc::new(move |admitted| {
                let selected = select(&runtime, admitted)?;
                let outcome = runtime
                    .request(&selected.principal, &selected.scope)
                    .on_branch(selected.branch)
                    .advanced()
                    .mutate(selected.intent)
                    .without_source()
                    .idempotency(&key_namespace)
                    .with_transport_idempotency(*admitted.scoped_key(), *admitted.stable_intent())
                    .execute_in_program(&runtime, ExecutionAllocationPolicy::SystemAllocation)?;
                Ok(match outcome {
                    Outcome::Committed { receipt, result } => Outcome::Committed {
                        receipt,
                        result: encode(result),
                    },
                    Outcome::AlreadyCommitted(receipt) => Outcome::AlreadyCommitted(receipt),
                    Outcome::RecoveredCommitted(receipt) => Outcome::RecoveredCommitted(receipt),
                    Outcome::IdempotencyIntentDrift => Outcome::IdempotencyIntentDrift,
                    Outcome::DomainDenied(denial) => Outcome::DomainDenied(encode(denial)),
                    Outcome::Cancelled => Outcome::Cancelled,
                    Outcome::DeadlineExceeded => Outcome::DeadlineExceeded,
                    Outcome::Commit(outcome) => Outcome::Commit(outcome),
                })
            }),
        }
    }
    pub(super) fn execute(
        &self,
        admitted: &WorthServerAdmittedQueryMutation,
    ) -> WorthServerQueryMutationAnswer {
        (self.execute)(admitted)
    }
}
fn encode(value: impl serde::Serialize) -> WorthServerQueryMutationPayload {
    serde_json::to_value(value).map_err(|error| error.to_string())
}
