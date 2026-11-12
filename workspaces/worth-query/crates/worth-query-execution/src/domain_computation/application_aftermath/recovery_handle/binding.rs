//! Immutable recovery binding derived from the committed receipt (R8.28).

use worth_query_installation::facade::{
    InstalledAftermathRecoveryContract, PublishedAftermathPosture,
    WorthQueryInstalledAftermathContract,
};
use worth_relational::facade::history::{BranchId, RelationalCommitReceipt};

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
    input_custody: RecoveryInputCustody,
    principal_scope: WorthQueryOperationScopeBinding,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    provider_posture: Option<WorthQueryExternalDispatchPosture>,
    recorded_provider_observation: Option<crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchRecordedObservation>,
    /// Co-committed outbox record — R8.28 correlation evidence (R8.68).
    committed_dispatch_outbox:
        Option<crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxBinding>,
    installed_aftermath: WorthQueryInstalledAftermathContract,
    expires_at_unix_ms: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RecoveryInputCustody {
    Performed {
        mutation_work: Option<WorthQueryPrimaryMutationWorkEvidence>,
        retained_preimage: Option<WorthQueryRetainedPreImage>,
        retained_governed_input: Option<WorthQueryRetainedGovernedInput>,
    },
    Recorded {
        governed_input_identity: Option<[u8; 32]>,
    },
}
impl RecoveryInputCustody {
    fn from_original(
        publication: &crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
        mutation_work: Option<WorthQueryPrimaryMutationWorkEvidence>,
        retained_preimage: Option<WorthQueryRetainedPreImage>,
        retained_governed_input: Option<WorthQueryRetainedGovernedInput>,
    ) -> Self {
        if publication
            .composite_commit()
            .owner_identity()
            .durable()
            .is_some()
        {
            Self::Recorded {
                governed_input_identity: retained_governed_input
                    .as_ref()
                    .and_then(|input| input.governed_identity()),
            }
        } else {
            Self::Performed {
                mutation_work,
                retained_preimage,
                retained_governed_input,
            }
        }
    }
}
impl WorthQueryRecoveryHandleBinding {
    pub(in crate::domain_computation) fn from_receipt(
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
            committed_product_publication: if receipt
                .committed_product_publication()
                .composite_commit()
                .owner_identity()
                .durable()
                .is_some()
            {
                receipt.committed_product_publication().descriptive()
            } else {
                receipt.committed_product_publication().clone()
            },
            application_binding_generation,
            installed_operation: *receipt.installed_operation(),
            input_custody: RecoveryInputCustody::from_original(
                receipt.committed_product_publication(),
                receipt.mutation_work().cloned(),
                receipt.retained_preimage().cloned(),
                receipt
                    .authority_binding()
                    .retained_governed_input_carrier()
                    .cloned(),
            ),
            principal_scope: receipt.principal_scope().clone(),
            idempotency: receipt.idempotency_binding(),
            provider_posture: receipt.external_dispatch().map(|d| d.posture().clone()),
            recorded_provider_observation: None,
            committed_dispatch_outbox: receipt.committed_dispatch_outbox().cloned(),
            installed_aftermath: aftermath.clone(),
            expires_at_unix_ms,
        })
    }

    pub(in crate::domain_computation::application_aftermath) fn from_pending_original(
        original: &crate::domain_computation::primary_graph::application_attempt::WorthQueryPendingApplicationCommitReceipt,
        dispatch: Option<
            &crate::domain_computation::application_aftermath::WorthQueryExternalEffectDispatch,
        >,
        expires_at_unix_ms: Option<u64>,
    ) -> Result<Self, WorthQueryRecoveryHandleDenial> {
        let authority = original.original_authority();
        let aftermath = authority.installed_aftermath().ok_or_else(|| {
            WorthQueryRecoveryHandleDenial::new(
                WorthQueryRecoveryHandleDenialKind::RecoveryNotAdmitted,
            )
        })?;
        match aftermath.recovery() {
            InstalledAftermathRecoveryContract::NotAdmitted => {
                return Err(WorthQueryRecoveryHandleDenial::new(
                    WorthQueryRecoveryHandleDenialKind::RecoveryNotAdmitted,
                ))
            }
            InstalledAftermathRecoveryContract::Admissible { .. } => {}
        }
        let scope = authority.principal_scope();
        Ok(Self {
            runtime_instance_id: original.original_runtime_instance_id(),
            schema_identity: *scope.binding_identity().schema_identity().bytes(),
            commit: original.original_commit().clone(),
            committed_product_publication: if original
                .original_publication()
                .composite_commit()
                .owner_identity()
                .durable()
                .is_some()
            {
                original.original_publication().descriptive()
            } else {
                original.original_publication().clone()
            },
            application_binding_generation: scope.binding_identity().generation(),
            installed_operation: *authority.installed_operation(),
            input_custody: RecoveryInputCustody::from_original(
                original.original_publication(),
                original.original_work().cloned(),
                original.original_preimage().cloned(),
                authority.retained_governed_input_carrier().cloned(),
            ),
            principal_scope: scope.clone(),
            idempotency: authority.idempotency_binding(),
            provider_posture: dispatch.map(|dispatch| dispatch.posture().clone()),
            recorded_provider_observation: None,
            committed_dispatch_outbox: original.original_outbox_binding().cloned(),
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

    /// A recovered observation is descriptive; it never becomes a live posture proof.
    pub fn provider_posture_kind(
        &self,
    ) -> Option<
        crate::domain_computation::application_aftermath::WorthQueryExternalDispatchPostureKind,
    > {
        use crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchRecordedObservation as Observation;
        use crate::domain_computation::application_aftermath::WorthQueryExternalDispatchPostureKind as Kind;
        self.provider_posture
            .as_ref()
            .map(|posture| posture.kind())
            .or_else(|| {
                self.recorded_provider_observation
                    .map(|observed| match observed {
                        Observation::Completed => Kind::Completed,
                        Observation::Acknowledged => Kind::Acknowledged,
                        Observation::ProvenNoEffect(_) => Kind::ProvenNoEffect,
                        Observation::Unresolved(_) => Kind::Unresolved,
                    })
            })
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

    pub(crate) fn governed_input_identity(&self) -> Option<[u8; 32]> {
        match &self.input_custody {
            RecoveryInputCustody::Performed {
                retained_governed_input,
                ..
            } => retained_governed_input
                .as_ref()
                .and_then(|input| input.governed_identity()),
            RecoveryInputCustody::Recorded {
                governed_input_identity,
            } => *governed_input_identity,
        }
    }

    pub(in crate::domain_computation) fn readmit_expiry(&mut self, expires_at_unix_ms: u64) {
        self.expires_at_unix_ms = Some(expires_at_unix_ms);
    }

    pub const fn expires_at_unix_ms(&self) -> Option<u64> {
        self.expires_at_unix_ms
    }
}

mod court_doors_for_test;

mod recorded;
