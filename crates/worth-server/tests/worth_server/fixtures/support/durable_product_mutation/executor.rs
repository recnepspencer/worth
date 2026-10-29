use std::sync::{atomic::Ordering, Arc, Mutex};

use serde_json::Value;
use worth_server::{
    WorthServerAdmittedDurableProductMutation, WorthServerDurableProductMutationCompletion,
    WorthServerDurableProductMutationConclusion, WorthServerDurableProductMutationExecution,
    WorthServerDurableProductMutationExecutor, WorthServerDurableProductMutationRecoveryHandle,
    WorthServerProductDurabilityCapability, WorthServerProductOperationBaseDigest,
    WorthServerProductOperationDenial, WorthServerProductOperationSuccess,
};

use super::persistence_state::{
    retained_record, retention_deadline, DurableProductState, DurableRecord, DurableScopeIdentity,
    DurableScopeState,
};
use super::product_result::DurableMutationProductResult;
use super::TestConcurrencyProbe;
#[path = "executor/crash_execution.rs"]
pub(crate) mod crash_execution;

use crash_execution::injected_crash_execution;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DurableMutationCrashPoint {
    BeforeIntent,
    AfterIntent,
    AfterMutationBeforeCommit,
    AfterCommitBeforeAcknowledgment,
}

impl DurableMutationCrashPoint {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BeforeIntent => "before-intent",
            Self::AfterIntent => "after-intent",
            Self::AfterMutationBeforeCommit => "after-mutation-before-commit",
            Self::AfterCommitBeforeAcknowledgment => "after-commit-before-acknowledgment",
        }
    }

    fn from_payload(payload: &Value) -> Option<Self> {
        match payload.get("crash_point").and_then(Value::as_str) {
            Some("before-intent") => Some(Self::BeforeIntent),
            Some("after-intent") => Some(Self::AfterIntent),
            Some("after-mutation-before-commit") => Some(Self::AfterMutationBeforeCommit),
            Some("after-commit-before-acknowledgment") => {
                Some(Self::AfterCommitBeforeAcknowledgment)
            }
            _ => None,
        }
    }
}

#[derive(Clone, Default)]
pub struct TestDurableProductExecutor {
    state: Arc<DurableProductState>,
    product_commit_namespace: Arc<str>,
}

impl TestDurableProductExecutor {
    pub fn with_product_commit_namespace(namespace: impl Into<Arc<str>>) -> Self {
        Self {
            state: Arc::default(),
            product_commit_namespace: namespace.into(),
        }
    }

    pub fn with_concurrency_probe(probe: TestConcurrencyProbe) -> Self {
        let executor = Self::default();
        *executor
            .state
            .concurrency_probe
            .lock()
            .expect("concurrency probe installation") = Some(probe);
        executor
    }

    pub fn commit_count(&self) -> usize {
        self.state.commit_count.load(Ordering::Relaxed)
    }

    pub fn observed_attempts(&self) -> Vec<(String, String)> {
        self.state
            .observed_attempts
            .lock()
            .expect("observed durable attempts")
            .clone()
    }

    pub fn advance_time(&self, seconds: u64) {
        self.state.now_seconds.fetch_add(seconds, Ordering::Relaxed);
    }

    pub fn completion_for(
        &self,
        tenant_id: &str,
        workspace_id: &str,
        authority_scope: &str,
        idempotency_key: &str,
    ) -> WorthServerDurableProductMutationCompletion {
        let identity = DurableScopeIdentity {
            tenant_id: tenant_id.to_string(),
            workspace_id: workspace_id.to_string(),
            authority_scope: authority_scope.to_string(),
        };
        self.state
            .scopes
            .lock()
            .expect("durable scope registry")
            .get(&identity)
            .expect("test scope should exist")
            .lock()
            .expect("durable product scope transaction")
            .records
            .get(idempotency_key)
            .expect("test completion should exist")
            .completion
            .clone()
    }

    pub fn override_recovery_with(&self, completion: WorthServerDurableProductMutationCompletion) {
        *self
            .state
            .recovery_override
            .lock()
            .expect("recovery override lock") = Some(completion);
    }

    fn execute_atomically(
        &self,
        attempt: &WorthServerAdmittedDurableProductMutation,
    ) -> WorthServerDurableProductMutationExecution {
        let scope = self.scope_state(attempt);
        let mut scope = scope.lock().expect("durable product scope transaction");
        let now = self.state.now_seconds.load(Ordering::Relaxed);
        let key = attempt.idempotency_key().value();
        if let Some(record) = retained_record(&mut scope.records, key, now) {
            return retry_conclusion(attempt, record);
        }

        let crash = DurableMutationCrashPoint::from_payload(attempt.payload().body());
        if self.consume_crash(
            &mut scope,
            key,
            crash,
            DurableMutationCrashPoint::BeforeIntent,
        ) {
            return injected_crash_execution(DurableMutationCrashPoint::BeforeIntent, false);
        }
        if self.consume_crash(
            &mut scope,
            key,
            crash,
            DurableMutationCrashPoint::AfterIntent,
        ) {
            return injected_crash_execution(DurableMutationCrashPoint::AfterIntent, false);
        }

        let current_basis = format!("basis:{}", scope.current_version);
        if attempt.expected_basis().base_digest().value() != current_basis {
            return WorthServerDurableProductMutationExecution::after_basis_comparison(
                WorthServerDurableProductMutationConclusion::StaleBasis {
                    observed_basis_digest: current_basis,
                },
            );
        }
        if attempt
            .payload()
            .body()
            .get("concurrency_probe")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            && !self.enter_concurrency_probe()
        {
            return WorthServerDurableProductMutationExecution::after_basis_comparison(
                WorthServerDurableProductMutationConclusion::failed(
                    "concurrency_probe_timeout",
                    "independent product scopes did not enter their transactions concurrently",
                ),
            );
        }

        let next_version = scope.current_version + 1;
        let completion =
            match build_completion(attempt, next_version, &self.product_commit_namespace) {
                Ok(completion) => completion,
                Err(error) => {
                    return WorthServerDurableProductMutationExecution::after_basis_comparison(
                        WorthServerDurableProductMutationConclusion::InvalidResultArtifact(error),
                    );
                }
            };
        if self.consume_crash(
            &mut scope,
            key,
            crash,
            DurableMutationCrashPoint::AfterMutationBeforeCommit,
        ) {
            return injected_crash_execution(
                DurableMutationCrashPoint::AfterMutationBeforeCommit,
                true,
            );
        }

        let record = DurableRecord {
            request_digest: attempt.request_digest().to_string(),
            completion: completion.clone(),
            retain_until: retention_deadline(attempt, now),
        };
        scope.current_version = next_version;
        scope.records.insert(key.to_string(), record.clone());
        self.state.commit_count.fetch_add(1, Ordering::Relaxed);

        if self.consume_crash(
            &mut scope,
            key,
            crash,
            DurableMutationCrashPoint::AfterCommitBeforeAcknowledgment,
        ) {
            let recovery = WorthServerDurableProductMutationRecoveryHandle::for_attempt(
                attempt,
                format!("recovery:{}", attempt.request_digest()),
            )
            .expect("test recovery handle should validate");
            self.state
                .recoveries
                .lock()
                .expect("durable recovery registry")
                .insert(recovery.canonical_digest().to_string(), record);
            return WorthServerDurableProductMutationExecution::after_basis_comparison(
                WorthServerDurableProductMutationConclusion::Indeterminate(recovery),
            );
        }

        WorthServerDurableProductMutationExecution::after_basis_comparison(
            WorthServerDurableProductMutationConclusion::Committed(completion),
        )
    }

    fn scope_state(
        &self,
        attempt: &WorthServerAdmittedDurableProductMutation,
    ) -> Arc<Mutex<DurableScopeState>> {
        let identity = DurableScopeIdentity {
            tenant_id: attempt.tenant_id().to_string(),
            workspace_id: attempt.workspace_id().to_string(),
            authority_scope: attempt.authority_scope().value().to_string(),
        };
        self.state
            .scopes
            .lock()
            .expect("durable scope registry")
            .entry(identity)
            .or_default()
            .clone()
    }

    fn consume_crash(
        &self,
        scope: &mut DurableScopeState,
        key: &str,
        actual: Option<DurableMutationCrashPoint>,
        expected: DurableMutationCrashPoint,
    ) -> bool {
        actual == Some(expected)
            && scope
                .consumed_crashes
                .insert(format!("{key}:{}", expected.as_str()))
    }

    fn enter_concurrency_probe(&self) -> bool {
        let probe = self
            .state
            .concurrency_probe
            .lock()
            .expect("concurrency probe installation")
            .clone();
        probe
            .as_ref()
            .is_none_or(TestConcurrencyProbe::enter_transaction)
    }
}

impl WorthServerDurableProductMutationExecutor for TestDurableProductExecutor {
    fn capability(&self) -> WorthServerProductDurabilityCapability {
        WorthServerProductDurabilityCapability::AtomicMutationCompletionV1
    }

    fn execute(
        &self,
        attempt: &WorthServerAdmittedDurableProductMutation,
    ) -> WorthServerDurableProductMutationExecution {
        self.state
            .observed_attempts
            .lock()
            .expect("observed durable attempts")
            .push((
                attempt.principal_id().to_string(),
                attempt.request_digest().to_string(),
            ));
        if let Some(reason_key) = attempt
            .payload()
            .body()
            .get("reject_reason")
            .and_then(Value::as_str)
        {
            return WorthServerDurableProductMutationExecution::after_basis_comparison(
                WorthServerDurableProductMutationConclusion::Rejected(
                    WorthServerProductOperationDenial::new(
                        reason_key,
                        "durable product mutation rejected",
                    ),
                ),
            );
        }
        self.execute_atomically(attempt)
    }

    fn resolve(
        &self,
        recovery: &WorthServerDurableProductMutationRecoveryHandle,
    ) -> WorthServerDurableProductMutationConclusion {
        if let Some(completion) = self
            .state
            .recovery_override
            .lock()
            .expect("recovery override lock")
            .clone()
        {
            return WorthServerDurableProductMutationConclusion::PreviouslyCommitted(completion);
        }
        let now = self.state.now_seconds.load(Ordering::Relaxed);
        let mut recoveries = self
            .state
            .recoveries
            .lock()
            .expect("durable recovery registry");
        retained_record(&mut recoveries, recovery.canonical_digest(), now)
            .map(|record| {
                WorthServerDurableProductMutationConclusion::PreviouslyCommitted(
                    record.completion.clone(),
                )
            })
            .unwrap_or_else(|| {
                WorthServerDurableProductMutationConclusion::failed(
                    "unknown_or_expired_recovery_handle",
                    "product persistence cannot resolve this recovery handle",
                )
            })
    }
}

fn retry_conclusion(
    attempt: &WorthServerAdmittedDurableProductMutation,
    record: &DurableRecord,
) -> WorthServerDurableProductMutationExecution {
    let conclusion = if record.request_digest == attempt.request_digest() {
        WorthServerDurableProductMutationConclusion::PreviouslyCommitted(record.completion.clone())
    } else {
        WorthServerDurableProductMutationConclusion::IdempotencyConflict {
            bound_request_digest: record.request_digest.clone(),
        }
    };
    WorthServerDurableProductMutationExecution::before_basis_comparison(conclusion)
}

fn build_completion(
    attempt: &WorthServerAdmittedDurableProductMutation,
    next_version: u64,
    product_commit_namespace: &str,
) -> Result<
    WorthServerDurableProductMutationCompletion,
    worth_server::WorthServerProductResultArtifactError,
> {
    let next_basis = WorthServerProductOperationBaseDigest::new(format!("basis:{next_version}"))
        .expect("test executor basis should be canonical");
    let result = DurableMutationProductResult::from_attempt(attempt, next_basis.value());
    let success = WorthServerProductOperationSuccess::publish_json(
        format!("{}:{next_version}", attempt.operation_name()),
        attempt.result_contract(),
        &result,
    )?;
    Ok(WorthServerDurableProductMutationCompletion::new(
        attempt,
        success,
        next_basis,
        format!(
            "product-commit:{product_commit_namespace}:{}:{next_version}",
            attempt.canonical_digest()
        ),
    )
    .expect("test executor completion should match its attempt"))
}
