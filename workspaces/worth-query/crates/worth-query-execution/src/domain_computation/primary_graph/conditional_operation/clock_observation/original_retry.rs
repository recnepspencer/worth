//! Original-key resolution and retry on a selected product branch.
//!
//! Routes directly through the existing provider/World idempotency result
//! owner without maintaining a second cache or re-executing the operation.
use super::WorthQueryConditionalClockObservationPort;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
use crate::domain_computation::primary_graph::conditional_operation::application_operation_reentry::{
    recovered_original, WorthQueryTemporalReentryOutcome,
};
use crate::domain_computation::primary_graph::provider::{
    WorthQueryConditionalKeyFate as Fate, WorthQueryProviderIdempotencyResolutionDenial as Denial,
    WorthQueryRecoveredApplicationCommitReceipt,
};
use crate::domain_computation::primary_graph::query_obligations::{
    QueryConditionalDecisionRecord, QueryConditionalOriginal,
};
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_query_installation::facade::ApplicationSchema;

/// Result of resolving or retrying an original conditional operation key on
/// the selected product branch through World's idempotency result owner.
#[derive(Debug)]
pub enum WorthQueryConditionalOriginalResolution {
    /// The original occurrence is published on this selected product branch.
    Published(WorthQueryRecoveredApplicationCommitReceipt),
    /// This live process still holds unpublished or pending publication
    /// custody for the original key; fresh execution stays fenced until World
    /// resolves it.
    Unresolved,
    /// World released the original keeping its owner effect.
    Released,
    /// No original exists on this selected product branch.
    Absent,
    /// Another root intent holds this key.
    OtherIntent,
}

impl<'runtime, Schema, Node, Clock>
    WorthQueryConditionalClockObservationPort<'runtime, Schema, Node, Clock>
where
    Schema: ApplicationSchema,
{
    /// Resolve or continue an original conditional operation key `(key, intent)`
    /// on this selected product branch through World's owning idempotency
    /// resolver, returning the original published occurrence without executing
    /// the operation again.
    pub fn retry_original_key(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        key: [u8; 32],
        intent: [u8; 32],
    ) -> Result<WorthQueryConditionalOriginalResolution, Denial> {
        let provider = self.runtime.inner.runtime.primary_provider_ref();
        let publication = self.truth.product().publication_binding();
        match provider.conditional_key_fate(execution, &publication, key, intent)? {
            Fate::Published(receipt) => {
                self.finish_conditional_decision_if_present(execution, key, intent);
                Ok(WorthQueryConditionalOriginalResolution::Published(receipt))
            }
            Fate::Released => {
                self.finish_conditional_decision_if_present(execution, key, intent);
                Ok(WorthQueryConditionalOriginalResolution::Released)
            }
            Fate::LiveUnpublishedOrPending => {
                Ok(WorthQueryConditionalOriginalResolution::Unresolved)
            }
            Fate::Absent => Ok(WorthQueryConditionalOriginalResolution::Absent),
            Fate::OtherIntent => Ok(WorthQueryConditionalOriginalResolution::OtherIntent),
            Fate::RecoveredUnpublished(_) => {
                let joined = self
                    .runtime
                    .installed_program_support()
                    .and_then(|support| {
                        match support.lifecycle().retained_indeterminate_original(&key) {
                            QueryConditionalOriginal::Recovered(original) => Some(original),
                            QueryConditionalOriginal::Unclassified => None,
                        }
                    });
                let binding = WorthQueryApplicationIdempotencyBinding::new(key, intent);
                match recovered_original::continue_recovered_original(
                    execution,
                    self.runtime,
                    self.truth.product(),
                    binding,
                    joined,
                ) {
                    Some(WorthQueryTemporalReentryOutcome::AlreadyCommitted(_)) => {
                        self.finish_conditional_decision_if_present(execution, key, intent);
                        match provider.conditional_key_fate(execution, &publication, key, intent)? {
                            Fate::Published(receipt) => {
                                Ok(WorthQueryConditionalOriginalResolution::Published(receipt))
                            }
                            Fate::Released => Ok(WorthQueryConditionalOriginalResolution::Released),
                            Fate::OtherIntent => {
                                Ok(WorthQueryConditionalOriginalResolution::OtherIntent)
                            }
                            Fate::Absent
                            | Fate::RecoveredUnpublished(_)
                            | Fate::LiveUnpublishedOrPending => Err(Denial::Unavailable),
                        }
                    }
                    _ => Err(Denial::Unavailable),
                }
            }
        }
    }

    /// Derive the original durable key for a temporal intent on this selected
    /// product branch using the installed temporal operation's binding and
    /// resolve/continue it through [`Self::retry_original_key`].
    pub fn retry_original_temporal_intent(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        intent_identity: &str,
        revision: u64,
        host_idempotency: &str,
        input_identity: &str,
    ) -> Result<WorthQueryConditionalOriginalResolution, Denial> {
        let derived = self.operation.lock_operation().temporal_original_key(
            self.truth.product(),
            intent_identity,
            revision,
            host_idempotency,
            input_identity,
        );
        let (key, intent) = derived
            .ok_or(Denial::Unavailable)?
            .map_err(|_| Denial::Unavailable)?;
        self.retry_original_key(execution, key, intent)
    }

    fn finish_conditional_decision_if_present(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        key: [u8; 32],
        intent: [u8; 32],
    ) {
        if let Some(support) = self.runtime.installed_program_support() {
            let _ = support.lifecycle().record_conditional_decision(
                execution,
                QueryConditionalDecisionRecord::Finish { key, intent },
            );
        }
        self.operation
            .lock_operation()
            .settle_resolved_continuation(key, intent);
    }
}
