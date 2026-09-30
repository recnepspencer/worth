use super::{
    WorthServerLoweredProductOperationPlan, WorthServerProductOperationDenial,
    WorthServerProductOperationEnvelope, WorthServerProductSchedulerAdmission,
    WorthServerScheduledProductOperation,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerProductOperationSuccess {
    result_key: String,
    result_artifact: crate::WorthServerProductResultArtifact,
}

impl WorthServerProductOperationSuccess {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        super::execution_pipeline::read_batch_accounting::string(&self.result_key)
            .saturating_add(self.result_artifact.owned_allocation_capacity_bytes())
    }

    pub fn publish_json<T>(
        result_key: impl Into<String>,
        contract: &crate::WorthServerProductResultContract,
        body: &T,
    ) -> Result<Self, crate::WorthServerProductResultArtifactError>
    where
        T: crate::WorthServerProductResultValue,
    {
        Ok(Self::from_artifact(
            result_key,
            crate::WorthServerProductResultArtifact::publish_json(contract, body)?,
        ))
    }

    pub(crate) fn from_artifact(
        result_key: impl Into<String>,
        result_artifact: crate::WorthServerProductResultArtifact,
    ) -> Self {
        Self {
            result_key: result_key.into(),
            result_artifact,
        }
    }

    pub fn result_key(&self) -> &str {
        &self.result_key
    }

    pub fn result_artifact(&self) -> &crate::WorthServerProductResultArtifact {
        &self.result_artifact
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerProductOperationFailure {
    reason_key: String,
    detail: String,
}

impl WorthServerProductOperationFailure {
    pub fn new(reason_key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            reason_key: reason_key.into(),
            detail: detail.into(),
        }
    }

    pub fn reason_key(&self) -> &str {
        &self.reason_key
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthServerProductOperationOutcome {
    Success(WorthServerProductOperationSuccess),
    Denied(WorthServerProductOperationDenial),
    Failed(WorthServerProductOperationFailure),
}

impl WorthServerProductOperationOutcome {
    pub fn failed(reason_key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::Failed(WorthServerProductOperationFailure::new(reason_key, detail))
    }
}

#[derive(Clone, Debug)]
pub struct WorthServerCompletedProductOperation {
    outcome: WorthServerProductOperationOutcome,
    envelope: WorthServerProductOperationEnvelope,
    proof: Option<WorthServerCompletedProductOperationProof>,
    adapter_execution_attempted: bool,
    durable_executor_attempted: bool,
    durable_mutation_receipt: Option<crate::WorthServerDurableProductMutationReceipt>,
    retry_receipt: Option<crate::WorthServerProductOperationRetryReceipt>,
}

impl WorthServerCompletedProductOperation {
    pub(crate) fn new(
        outcome: WorthServerProductOperationOutcome,
        envelope: WorthServerProductOperationEnvelope,
    ) -> Self {
        Self {
            outcome,
            envelope,
            proof: None,
            adapter_execution_attempted: false,
            durable_executor_attempted: false,
            durable_mutation_receipt: None,
            retry_receipt: None,
        }
    }

    pub(crate) fn with_scheduled_operation(
        mut self,
        scheduled_operation: &WorthServerScheduledProductOperation,
    ) -> Self {
        self.proof = Some(WorthServerCompletedProductOperationProof::new(
            scheduled_operation.plan().clone(),
            scheduled_operation.scheduler_admission().clone(),
        ));
        self.adapter_execution_attempted = true;
        self
    }

    pub(crate) fn with_durable_executor_attempt(
        mut self,
        scheduled_operation: &WorthServerScheduledProductOperation,
    ) -> Self {
        self.proof = Some(WorthServerCompletedProductOperationProof::new(
            scheduled_operation.plan().clone(),
            scheduled_operation.scheduler_admission().clone(),
        ));
        self.adapter_execution_attempted = false;
        self.durable_executor_attempted = true;
        self
    }

    pub(crate) fn with_retry_receipt(
        mut self,
        retry_receipt: crate::WorthServerProductOperationRetryReceipt,
    ) -> Self {
        self.retry_receipt = Some(retry_receipt);
        self
    }

    pub(crate) fn with_durable_mutation_receipt(
        mut self,
        durable_mutation_receipt: crate::WorthServerDurableProductMutationReceipt,
    ) -> Self {
        self.durable_mutation_receipt = Some(durable_mutation_receipt);
        self
    }

    pub(crate) fn as_previously_committed(
        &self,
        retry_receipt: crate::WorthServerProductOperationRetryReceipt,
    ) -> Self {
        let mut cloned = self.clone();
        cloned.adapter_execution_attempted = false;
        cloned.durable_executor_attempted = false;
        cloned.retry_receipt = Some(retry_receipt);
        cloned
    }

    pub fn outcome(&self) -> &WorthServerProductOperationOutcome {
        &self.outcome
    }

    pub fn envelope(&self) -> &WorthServerProductOperationEnvelope {
        &self.envelope
    }

    pub fn result_artifact(&self) -> Option<&crate::WorthServerProductResultArtifact> {
        match &self.outcome {
            WorthServerProductOperationOutcome::Success(success) => Some(success.result_artifact()),
            WorthServerProductOperationOutcome::Denied(_)
            | WorthServerProductOperationOutcome::Failed(_) => None,
        }
    }

    pub fn plan(&self) -> Option<&WorthServerLoweredProductOperationPlan> {
        self.proof
            .as_ref()
            .map(WorthServerCompletedProductOperationProof::plan)
    }

    pub fn scheduler_admission(&self) -> Option<&WorthServerProductSchedulerAdmission> {
        self.proof
            .as_ref()
            .map(WorthServerCompletedProductOperationProof::scheduler_admission)
    }

    pub fn support_posture(&self) -> Option<&crate::WorthServerOperationSupportPosture> {
        self.plan()
            .map(WorthServerLoweredProductOperationPlan::support_posture)
    }

    pub fn precondition_posture(&self) -> Option<&crate::WorthServerOperationPreconditionPosture> {
        self.plan()
            .map(WorthServerLoweredProductOperationPlan::precondition_posture)
    }

    pub fn retry_receipt(&self) -> Option<&crate::WorthServerProductOperationRetryReceipt> {
        self.retry_receipt.as_ref()
    }

    pub fn durable_mutation_receipt(
        &self,
    ) -> Option<&crate::WorthServerDurableProductMutationReceipt> {
        self.durable_mutation_receipt.as_ref()
    }

    pub fn retry_diagnostics(&self) -> WorthServerProductOperationRetryDiagnostics {
        WorthServerProductOperationRetryDiagnostics::new(
            self.retry_receipt.clone(),
            self.adapter_execution_attempted(),
            self.envelope.canonical_digest().to_string(),
        )
    }

    pub fn adapter_execution_attempted(&self) -> bool {
        self.adapter_execution_attempted
    }

    pub fn durable_executor_attempted(&self) -> bool {
        self.durable_executor_attempted
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthServerProductOperationRetryClass {
    BestEffort,
    Executed,
    PreviouslyCommitted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerProductOperationRetryDiagnostics {
    class: WorthServerProductOperationRetryClass,
    retry_receipt: Option<crate::WorthServerProductOperationRetryReceipt>,
    adapter_execution_attempted: bool,
    envelope_digest: String,
}

impl WorthServerProductOperationRetryDiagnostics {
    fn new(
        retry_receipt: Option<crate::WorthServerProductOperationRetryReceipt>,
        adapter_execution_attempted: bool,
        envelope_digest: String,
    ) -> Self {
        let class = match retry_receipt.as_ref() {
            None => WorthServerProductOperationRetryClass::BestEffort,
            Some(receipt) if receipt.is_previously_committed() => {
                WorthServerProductOperationRetryClass::PreviouslyCommitted
            }
            Some(_) => WorthServerProductOperationRetryClass::Executed,
        };
        Self {
            class,
            retry_receipt,
            adapter_execution_attempted,
            envelope_digest,
        }
    }

    pub fn class(&self) -> &WorthServerProductOperationRetryClass {
        &self.class
    }

    pub fn retry_receipt(&self) -> Option<&crate::WorthServerProductOperationRetryReceipt> {
        self.retry_receipt.as_ref()
    }

    pub fn is_executed(&self) -> bool {
        self.class == WorthServerProductOperationRetryClass::Executed
    }

    pub fn is_previously_committed(&self) -> bool {
        self.class == WorthServerProductOperationRetryClass::PreviouslyCommitted
    }

    pub fn adapter_execution_attempted(&self) -> bool {
        self.adapter_execution_attempted
    }

    pub fn adapter_execution_skipped_by_retry(&self) -> bool {
        self.is_previously_committed() && !self.adapter_execution_attempted
    }

    pub fn envelope_digest(&self) -> &str {
        &self.envelope_digest
    }
}

#[derive(Clone, Debug)]
struct WorthServerCompletedProductOperationProof {
    plan: WorthServerLoweredProductOperationPlan,
    scheduler_admission: WorthServerProductSchedulerAdmission,
}

impl WorthServerCompletedProductOperationProof {
    fn new(
        plan: WorthServerLoweredProductOperationPlan,
        scheduler_admission: WorthServerProductSchedulerAdmission,
    ) -> Self {
        Self {
            plan,
            scheduler_admission,
        }
    }

    fn plan(&self) -> &WorthServerLoweredProductOperationPlan {
        &self.plan
    }

    fn scheduler_admission(&self) -> &WorthServerProductSchedulerAdmission {
        &self.scheduler_admission
    }
}
