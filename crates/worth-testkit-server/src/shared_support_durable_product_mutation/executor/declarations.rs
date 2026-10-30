pub use {std::sync::atomic::Ordering, std::sync::Arc, std::sync::Mutex};

pub use serde_json::Value;
pub use {
    worth_server::WorthServerAdmittedDurableProductMutation,
    worth_server::WorthServerDurableProductMutationCompletion,
    worth_server::WorthServerDurableProductMutationConclusion,
    worth_server::WorthServerDurableProductMutationExecution,
    worth_server::WorthServerDurableProductMutationExecutor,
    worth_server::WorthServerDurableProductMutationRecoveryHandle,
    worth_server::WorthServerProductDurabilityCapability,
    worth_server::WorthServerProductOperationBaseDigest,
    worth_server::WorthServerProductOperationDenial,
    worth_server::WorthServerProductOperationSuccess,
};

pub use crate::shared_support_durable_product_mutation::concurrency_probe::TestConcurrencyProbe;
pub use crate::shared_support_durable_product_mutation::product_result::DurableMutationProductResult;
pub use {
    crate::shared_support_durable_product_mutation::persistence_state::retained_record,
    crate::shared_support_durable_product_mutation::persistence_state::retention_deadline,
    crate::shared_support_durable_product_mutation::persistence_state::DurableProductState,
    crate::shared_support_durable_product_mutation::persistence_state::DurableRecord,
    crate::shared_support_durable_product_mutation::persistence_state::DurableScopeIdentity,
    crate::shared_support_durable_product_mutation::persistence_state::DurableScopeState,
};

pub use crate::shared_support_durable_product_mutation::executor::crash_execution::injected_crash_execution;

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

    pub fn from_payload(payload: &Value) -> Option<Self> {
        match payload
            .get("crash_point")
            .and_then(serde_json::Value::as_str)
        {
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
    pub state: Arc<DurableProductState>,
    pub product_commit_namespace: Arc<str>,
}

impl TestDurableProductExecutor {
    pub fn with_product_commit_namespace(namespace: impl Into<Arc<str>>) -> Self {
        Self {
            state: std::sync::Arc::default(),
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
        self.state
            .commit_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn observed_attempts(&self) -> Vec<(String, String)> {
        self.state
            .observed_attempts
            .lock()
            .expect("observed durable attempts")
            .clone()
    }

    pub fn advance_time(&self, seconds: u64) {
        self.state
            .now_seconds
            .fetch_add(seconds, std::sync::atomic::Ordering::Relaxed);
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

    pub fn execute_atomically(
        &self,
        attempt: &WorthServerAdmittedDurableProductMutation,
    ) -> WorthServerDurableProductMutationExecution {
        let scope = self.scope_state(attempt);
        let mut scope = scope.lock().expect("durable product scope transaction");
        let now = self
            .state
            .now_seconds
            .load(std::sync::atomic::Ordering::Relaxed);
        let key = attempt.idempotency_key().value();
        if let Some(record) = retained_record(&mut scope.records, key, now) {
            return retry_conclusion(attempt, record);
        }

        let crash = crate::shared_support_durable_product_mutation::executor::DurableMutationCrashPoint::from_payload(attempt.payload().body());
        if self.consume_crash(
            &mut scope,
            key,
            crash,
            crate::shared_support_durable_product_mutation::executor::DurableMutationCrashPoint::BeforeIntent,
        ) {
            return injected_crash_execution(crate::shared_support_durable_product_mutation::executor::DurableMutationCrashPoint::BeforeIntent, false);
        }
        if self.consume_crash(
            &mut scope,
            key,
            crash,
            crate::shared_support_durable_product_mutation::executor::DurableMutationCrashPoint::AfterIntent,
        ) {
            return injected_crash_execution(crate::shared_support_durable_product_mutation::executor::DurableMutationCrashPoint::AfterIntent, false);
        }

        let current_basis = format!("basis:{}", scope.current_version);
        if attempt.expected_basis().base_digest().value() != current_basis {
            return worth_server::WorthServerDurableProductMutationExecution::after_basis_comparison(
                worth_server::WorthServerDurableProductMutationConclusion::StaleBasis {
                    observed_basis_digest: current_basis,
                },
            );
        }
        if attempt
            .payload()
            .body()
            .get("concurrency_probe")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
            && !self.enter_concurrency_probe()
        {
            return worth_server::WorthServerDurableProductMutationExecution::after_basis_comparison(
                worth_server::WorthServerDurableProductMutationConclusion::failed(
                    "concurrency_probe_timeout",
                    "independent product scopes did not enter their transactions concurrently",
                ),
            );
        }

        let next_version = scope.current_version + 1;
        let completion = match build_completion(
            attempt,
            next_version,
            &self.product_commit_namespace,
        ) {
            Ok(completion) => completion,
            Err(error) => {
                return worth_server::WorthServerDurableProductMutationExecution::after_basis_comparison(
                        worth_server::WorthServerDurableProductMutationConclusion::InvalidResultArtifact(error),
                    );
            }
        };
        if self.consume_crash(
            &mut scope,
            key,
            crash,
            crate::shared_support_durable_product_mutation::executor::DurableMutationCrashPoint::AfterMutationBeforeCommit,
        ) {
            return injected_crash_execution(
                crate::shared_support_durable_product_mutation::executor::DurableMutationCrashPoint::AfterMutationBeforeCommit,
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
        self.state
            .commit_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        if self.consume_crash(
            &mut scope,
            key,
            crash,
            crate::shared_support_durable_product_mutation::executor::DurableMutationCrashPoint::AfterCommitBeforeAcknowledgment,
        ) {
            let recovery = worth_server::WorthServerDurableProductMutationRecoveryHandle::for_attempt(
                attempt,
                format!("recovery:{}", attempt.request_digest()),
            )
            .expect("test recovery handle should validate");
            self.state
                .recoveries
                .lock()
                .expect("durable recovery registry")
                .insert(recovery.canonical_digest().to_string(), record);
            return worth_server::WorthServerDurableProductMutationExecution::after_basis_comparison(
                worth_server::WorthServerDurableProductMutationConclusion::Indeterminate(recovery),
            );
        }

        worth_server::WorthServerDurableProductMutationExecution::after_basis_comparison(
            worth_server::WorthServerDurableProductMutationConclusion::Committed(completion),
        )
    }

    pub fn scope_state(
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

    pub fn consume_crash(
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

    pub fn enter_concurrency_probe(&self) -> bool {
        let probe = self
            .state
            .concurrency_probe
            .lock()
            .expect("concurrency probe installation")
            .clone();
        probe
            .as_ref()
            .is_none_or(crate::shared_support_durable_product_mutation::concurrency_probe::TestConcurrencyProbe::enter_transaction)
    }
}

impl WorthServerDurableProductMutationExecutor for TestDurableProductExecutor {
    fn capability(&self) -> WorthServerProductDurabilityCapability {
        worth_server::WorthServerProductDurabilityCapability::AtomicMutationCompletionV1
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
            .and_then(serde_json::Value::as_str)
        {
            return worth_server::WorthServerDurableProductMutationExecution::after_basis_comparison(
                worth_server::WorthServerDurableProductMutationConclusion::Rejected(
                    worth_server::WorthServerProductOperationDenial::new(
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
            return worth_server::WorthServerDurableProductMutationConclusion::PreviouslyCommitted(
                completion,
            );
        }
        let now = self
            .state
            .now_seconds
            .load(std::sync::atomic::Ordering::Relaxed);
        let mut recoveries = self
            .state
            .recoveries
            .lock()
            .expect("durable recovery registry");
        retained_record(&mut recoveries, recovery.canonical_digest(), now)
            .map(|record| {
                worth_server::WorthServerDurableProductMutationConclusion::PreviouslyCommitted(
                    record.completion.clone(),
                )
            })
            .unwrap_or_else(|| {
                worth_server::WorthServerDurableProductMutationConclusion::failed(
                    "unknown_or_expired_recovery_handle",
                    "product persistence cannot resolve this recovery handle",
                )
            })
    }
}

#[path = "declarations/result_completion.rs"]
mod result_completion;
pub use result_completion::*;
