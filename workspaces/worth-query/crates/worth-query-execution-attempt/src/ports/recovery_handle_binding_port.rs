//! Crate-boundary internal port for `WorthQueryRecoveryHandleBinding`.
//! Never exported on `worth-query-execution::facade` or Host.

use crate::domain_computation::application_aftermath::recovery_handle::{
    WorthQueryRecoveryHandleBinding, WorthQueryRecoveryHandleDenial,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationIdempotencyBinding,
};

pub trait RecoveryHandleBindingInternalPort: Sized {
    fn from_receipt(
        receipt: &WorthQueryApplicationCommitReceipt,
        expires_at_unix_ms: Option<u64>,
    ) -> Result<Self, WorthQueryRecoveryHandleDenial>;

    fn from_pending_original(
        original: &crate::domain_computation::primary_graph::application_attempt::WorthQueryPendingApplicationCommitReceipt,
        dispatch: Option<
            &crate::domain_computation::application_aftermath::WorthQueryExternalEffectDispatch,
        >,
        expires_at_unix_ms: Option<u64>,
    ) -> Result<Self, WorthQueryRecoveryHandleDenial>;

    #[allow(clippy::too_many_arguments)]
    fn from_readmitted<
        Schema: worth_query_installation::facade::ApplicationSchema,
        Operation,
        Input,
        Scope,
    >(
        runtime_instance_id: u64,
        admission: &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            Scope,
        >,
        publication: crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        outbox: Option<
            crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxBinding,
        >,
        expires_at_unix_ms: u64,
        recorded_provider_observation: Option<
            crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchRecordedObservation,
        >,
    ) -> Result<Self, WorthQueryRecoveryHandleDenial>;

    fn committed_dispatch_outbox(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxBinding>;

    fn installed_aftermath(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledAftermathContract;

    fn governed_input_identity(&self) -> Option<[u8; 32]>;

    fn readmit_expiry(&mut self, expires_at_unix_ms: u64);
}

impl RecoveryHandleBindingInternalPort for WorthQueryRecoveryHandleBinding {
    fn from_receipt(
        receipt: &WorthQueryApplicationCommitReceipt,
        expires_at_unix_ms: Option<u64>,
    ) -> Result<Self, WorthQueryRecoveryHandleDenial> {
        Self::from_receipt(receipt, expires_at_unix_ms)
    }

    fn from_pending_original(
        original: &crate::domain_computation::primary_graph::application_attempt::WorthQueryPendingApplicationCommitReceipt,
        dispatch: Option<
            &crate::domain_computation::application_aftermath::WorthQueryExternalEffectDispatch,
        >,
        expires_at_unix_ms: Option<u64>,
    ) -> Result<Self, WorthQueryRecoveryHandleDenial> {
        Self::from_pending_original(original, dispatch, expires_at_unix_ms)
    }

    fn from_readmitted<
        Schema: worth_query_installation::facade::ApplicationSchema,
        Operation,
        Input,
        Scope,
    >(
        runtime_instance_id: u64,
        admission: &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            Scope,
        >,
        publication: crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        outbox: Option<
            crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxBinding,
        >,
        expires_at_unix_ms: u64,
        recorded_provider_observation: Option<
            crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchRecordedObservation,
        >,
    ) -> Result<Self, WorthQueryRecoveryHandleDenial> {
        Self::from_readmitted(
            runtime_instance_id,
            admission,
            publication,
            idempotency,
            outbox,
            expires_at_unix_ms,
            recorded_provider_observation,
        )
    }

    fn committed_dispatch_outbox(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxBinding>
    {
        Self::committed_dispatch_outbox(self)
    }

    fn installed_aftermath(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledAftermathContract {
        Self::installed_aftermath(self)
    }

    fn governed_input_identity(&self) -> Option<[u8; 32]> {
        Self::governed_input_identity(self)
    }

    fn readmit_expiry(&mut self, expires_at_unix_ms: u64) {
        Self::readmit_expiry(self, expires_at_unix_ms)
    }
}
