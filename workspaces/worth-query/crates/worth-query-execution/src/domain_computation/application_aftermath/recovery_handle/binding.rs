//! Immutable recovery binding derived from the committed receipt (R8.28).

use worth_query_installation::facade::{
    InstalledAftermathRecoveryContract, PublishedAftermathPosture,
    WorthQueryInstalledAftermathContract,
};
#[cfg(test)]
use worth_relational::facade::history::CommitId;
use worth_relational::facade::history::{BranchId, RelationalCommitReceipt};
#[cfg(test)]
use worth_relational::facade::identity::VersionId;

use crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage;
use crate::domain_computation::application_aftermath::{
    ExternalEffectCorrelationIdentity, WorthQueryDispatchOutboxRecord,
    WorthQueryExternalDispatchPosture,
};
use crate::domain_computation::authorization::WorthQueryOperationScopeBinding;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationIdempotencyBinding,
    WorthQueryPrimaryMutationWorkEvidence, WorthQueryRetainedGovernedInput,
};

use super::denial::{WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind};

/// Binding axes a recovery handle retains from admitted commit truth.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRecoveryHandleBinding {
    runtime_instance_id: u64,
    schema_identity: [u8; 32],
    commit: RelationalCommitReceipt,
    committed_product_publication:
        crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
    /// Generation of the installed application schema binding.
    application_binding_generation: u64,
    installed_operation: [u8; 32],
    mutation_work: Option<WorthQueryPrimaryMutationWorkEvidence>,
    retained_preimage: Option<WorthQueryRetainedPreImage>,
    retained_governed_input: Option<WorthQueryRetainedGovernedInput>,
    principal_scope: WorthQueryOperationScopeBinding,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    provider_posture: Option<WorthQueryExternalDispatchPosture>,
    /// Co-committed outbox record — R8.28 correlation evidence (R8.68).
    committed_dispatch_outbox:
        Option<crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxBinding>,
    installed_aftermath: WorthQueryInstalledAftermathContract,
    expires_at_unix_ms: Option<u64>,
}

impl WorthQueryRecoveryHandleBinding {
    pub(super) fn from_receipt(
        receipt: &WorthQueryApplicationCommitReceipt,
        expires_at_unix_ms: Option<u64>,
    ) -> Result<Self, WorthQueryRecoveryHandleDenial> {
        let aftermath = receipt.installed_aftermath().ok_or_else(|| {
            WorthQueryRecoveryHandleDenial::new(
                WorthQueryRecoveryHandleDenialKind::RecoveryNotAdmitted,
            )
        })?;
        match aftermath.recovery() {
            InstalledAftermathRecoveryContract::NotAdmitted => {
                return Err(WorthQueryRecoveryHandleDenial::new(
                    WorthQueryRecoveryHandleDenialKind::RecoveryNotAdmitted,
                ));
            }
            InstalledAftermathRecoveryContract::Admissible { .. } => {}
        }
        let schema_identity = *receipt
            .principal_scope()
            .binding_identity()
            .schema_identity()
            .bytes();
        let commit = receipt.commit_reference().clone();
        let application_binding_generation =
            receipt.principal_scope().binding_identity().generation();
        Ok(Self {
            runtime_instance_id: receipt.provider_runtime_instance_id(),
            schema_identity,
            commit,
            committed_product_publication: receipt.committed_product_publication().clone(),
            application_binding_generation,
            installed_operation: *receipt.installed_operation(),
            mutation_work: receipt.mutation_work().cloned(),
            retained_preimage: receipt.retained_preimage().cloned(),
            retained_governed_input: receipt
                .authority_binding()
                .retained_governed_input_carrier()
                .cloned(),
            principal_scope: receipt.principal_scope().clone(),
            idempotency: receipt.idempotency_binding(),
            provider_posture: receipt.external_dispatch().map(|d| d.posture().clone()),
            committed_dispatch_outbox: receipt.committed_dispatch_outbox().cloned(),
            installed_aftermath: aftermath.clone(),
            expires_at_unix_ms,
        })
    }

    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub const fn schema_identity(&self) -> &[u8; 32] {
        &self.schema_identity
    }

    pub const fn branch(&self) -> &BranchId {
        &self.commit.branch_id
    }

    pub const fn application_binding_generation(&self) -> u64 {
        self.application_binding_generation
    }

    pub const fn installed_operation(&self) -> &[u8; 32] {
        &self.installed_operation
    }

    pub const fn attempt_commit_id(&self) -> u64 {
        self.commit.commit_id.0
    }

    /// Exact Relational commit identity retained from the ordinary commit.
    pub const fn commit_reference(&self) -> &RelationalCommitReceipt {
        &self.commit
    }

    pub const fn committed_product_publication(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication {
        &self.committed_product_publication
    }

    pub const fn principal_scope(&self) -> &WorthQueryOperationScopeBinding {
        &self.principal_scope
    }

    pub const fn idempotency(&self) -> WorthQueryApplicationIdempotencyBinding {
        self.idempotency
    }

    pub const fn provider_posture(&self) -> Option<&WorthQueryExternalDispatchPosture> {
        self.provider_posture.as_ref()
    }

    /// Correlation identity derived from the bound outbox record.
    ///
    /// Preserved for callers that only need the identity; re-dispatch requires
    /// [`Self::dispatch_outbox`] (R8.68).
    pub fn correlation(&self) -> Option<ExternalEffectCorrelationIdentity> {
        self.committed_dispatch_outbox
            .as_ref()
            .map(|binding| *binding.record().correlation())
    }

    pub fn dispatch_outbox(&self) -> Option<&WorthQueryDispatchOutboxRecord> {
        self.committed_dispatch_outbox
            .as_ref()
            .map(crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxBinding::record)
    }

    pub(in crate::domain_computation) fn committed_dispatch_outbox(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxBinding>
    {
        self.committed_dispatch_outbox.as_ref()
    }

    pub fn compatibility_generation(&self) -> u64 {
        self.installed_aftermath.compatibility_generation()
    }

    pub fn published_posture(&self) -> PublishedAftermathPosture {
        self.installed_aftermath.published_posture()
    }

    /// Identity of the exact installed aftermath retained through commit.
    pub const fn installed_aftermath_identity(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledAftermathIdentity {
        self.installed_aftermath.identity()
    }

    /// Operation slot owned by that same installed aftermath contract.
    pub fn installed_aftermath_operation_slot(&self) -> &str {
        self.installed_aftermath.operation_slot()
    }

    pub(crate) const fn installed_aftermath(&self) -> &WorthQueryInstalledAftermathContract {
        &self.installed_aftermath
    }

    pub(crate) const fn mutation_work(&self) -> Option<&WorthQueryPrimaryMutationWorkEvidence> {
        self.mutation_work.as_ref()
    }

    pub(crate) const fn retained_preimage(&self) -> Option<&WorthQueryRetainedPreImage> {
        self.retained_preimage.as_ref()
    }

    pub(crate) const fn retained_governed_input(&self) -> Option<&WorthQueryRetainedGovernedInput> {
        self.retained_governed_input.as_ref()
    }

    /// Exact original capability input carried from operation admission.
    pub fn original_input<Input: 'static>(&self) -> Option<&Input> {
        self.retained_governed_input.as_ref()?.downcast_ref()
    }

    pub const fn expires_at_unix_ms(&self) -> Option<u64> {
        self.expires_at_unix_ms
    }
}

/// Test-only editor whose baseline is a production World commit.
///
/// Each method corrupts one named axis for a negative proof. `finish` cannot
/// create the World publication identity or its Relational pairing.
#[cfg(test)]
pub(crate) struct WorthQueryRecoveryHandleBindingAxisProbe {
    binding: WorthQueryRecoveryHandleBinding,
}

#[cfg(test)]
impl WorthQueryRecoveryHandleBindingAxisProbe {
    pub(crate) fn real() -> Self {
        static BASELINE: std::sync::OnceLock<WorthQueryRecoveryHandleBinding> =
            std::sync::OnceLock::new();
        let binding = BASELINE
            .get_or_init(|| {
                let receipt =
                    crate::domain_computation::primary_graph::committed_recoverable_application();
                WorthQueryRecoveryHandleBinding::from_receipt(&receipt, Some(u64::MAX))
                    .expect("the real fixture operation admits recovery")
            })
            .clone();
        Self { binding }
    }

    pub(crate) fn from_binding(binding: WorthQueryRecoveryHandleBinding) -> Self {
        Self { binding }
    }

    pub(crate) fn runtime_instance_id(mut self, value: u64) -> Self {
        self.binding.runtime_instance_id = value;
        self
    }

    pub(crate) fn schema_identity(mut self, value: [u8; 32]) -> Self {
        self.binding.schema_identity = value;
        self
    }

    pub(crate) fn branch(mut self, value: BranchId) -> Self {
        self.binding.commit.branch_id = value;
        self
    }

    pub(crate) fn attempt_commit_id(mut self, value: u64) -> Self {
        self.binding.commit.commit_id = CommitId(value);
        self.binding.commit.version_id = VersionId(value);
        self.binding.commit.parents.clear();
        self
    }

    pub(crate) fn application_binding_generation(mut self, value: u64) -> Self {
        self.binding.application_binding_generation = value;
        self
    }

    pub(crate) fn installed_operation(mut self, value: [u8; 32]) -> Self {
        self.binding.installed_operation = value;
        self
    }

    pub(crate) fn retained_governed_input_identity(mut self, value: Option<[u8; 32]>) -> Self {
        self.binding.retained_governed_input =
            value.map(WorthQueryRetainedGovernedInput::axis_probe);
        self
    }

    pub(crate) fn principal_scope(mut self, value: WorthQueryOperationScopeBinding) -> Self {
        self.binding.principal_scope = value;
        self
    }

    pub(crate) fn idempotency(mut self, value: WorthQueryApplicationIdempotencyBinding) -> Self {
        self.binding.idempotency = value;
        self
    }

    pub(crate) fn installed_aftermath(
        mut self,
        value: WorthQueryInstalledAftermathContract,
    ) -> Self {
        self.binding.installed_aftermath = value;
        self
    }

    pub(crate) fn without_dispatch_outbox(mut self) -> Self {
        self.binding.committed_dispatch_outbox = None;
        self
    }

    pub(crate) fn expires_at_unix_ms(mut self, value: Option<u64>) -> Self {
        self.binding.expires_at_unix_ms = value;
        self
    }

    pub(crate) fn finish(self) -> WorthQueryRecoveryHandleBinding {
        self.binding
    }
}
