use crate::authorization::{
    consume_authorization_through, record_recovery_staging_completion,
    recover_authorization_consumption, StagingAuthorizationContinuation,
};
use crate::workflow::persist_recovery_owner_receipts;
use crate::{
    AuthorizationConsumptionDenial, AuthorizationConsumptionReceipt,
    AuthorizationRevocationObservation, OperationalControlStore, OperationalTransitionId,
};
use worth_store_authority::StoreCurrentAuthorityWitness;
use worth_store_physical_backend::{
    NonCurrentStagingExecutionDenial, NonCurrentStagingExecutionReceipt,
    NonCurrentStagingOwnerExecutionDenial, PhysicalRecoveryStagingOwner,
};

use super::{
    AuthorizedPointInTimeRecoveryPlan, LoweredPointInTimeRecoveryPlan, PointInTimeRecoveryReceipt,
    PointInTimeReplayDenial, PointInTimeReplayOwner, PointInTimeReplayPlan,
};

#[derive(Debug)]
pub enum PitrReadinessDenial {
    StaleAuthority,
    Target(crate::control_store::NonCurrentRecoveryTargetDenial),
    Authorization(AuthorizationConsumptionDenial),
}

pub struct ExecutionReadyPointInTimeRecovery<'a> {
    operation_id: crate::OperationalOperationId,
    authorization: AuthorizationConsumptionReceipt,
    staging_authority: worth_store_authority::StoreCurrentAuthorityIdentity,
    security_scope: crate::OperationalSecurityScope,
    backend: worth_store_physical_backend::LoweredNonCurrentStagingPlan,
    recovery: PointInTimeReplayPlan,
    lease: worth_store_physical_isolation::PitrReachabilityLease,
    owner_verification: worth_store_offline_verifier::StagedRecoveryOwnerVerificationSet,
    _target_admission: crate::control_store::NonCurrentRecoveryTargetAdmission,
    control: &'a dyn crate::OperationalControlStorePort,
}

impl std::fmt::Debug for ExecutionReadyPointInTimeRecovery<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ExecutionReadyPointInTimeRecovery")
            .field("operation_id", &self.operation_id)
            .field("staging_authority", &self.staging_authority)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub enum PitrExecutionDenial {
    Authorization(crate::StagingAuthorizationContinuationDenial),
    Backend(NonCurrentStagingExecutionDenial),
    Recovery(PointInTimeReplayDenial),
    Control(crate::OperationalControlAppendDenial),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointInTimeRecoveryOperationReceipt {
    authorization: AuthorizationConsumptionReceipt,
    backend: NonCurrentStagingExecutionReceipt,
    recovery: PointInTimeRecoveryReceipt,
}

impl PointInTimeRecoveryOperationReceipt {
    pub const fn authorization(&self) -> AuthorizationConsumptionReceipt {
        self.authorization
    }
    pub const fn backend(&self) -> &NonCurrentStagingExecutionReceipt {
        &self.backend
    }
    pub const fn recovery(&self) -> PointInTimeRecoveryReceipt {
        self.recovery
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedPointInTimeRecovery {
    operation_id: crate::OperationalOperationId,
    receipt: PointInTimeRecoveryOperationReceipt,
    staging_authority: worth_store_authority::StoreCurrentAuthorityIdentity,
    security_scope: crate::OperationalSecurityScope,
    source_lease: worth_store_physical_isolation::PitrReachabilityLease,
    owner_verification: worth_store_offline_verifier::StagedRecoveryOwnerVerificationSet,
}

impl ExecutedPointInTimeRecovery {
    pub const fn operation_id(&self) -> &crate::OperationalOperationId {
        &self.operation_id
    }
    pub const fn staging_authority(&self) -> worth_store_authority::StoreCurrentAuthorityIdentity {
        self.staging_authority
    }
    pub const fn security_scope(&self) -> crate::OperationalSecurityScope {
        self.security_scope
    }
    pub const fn receipt(&self) -> &PointInTimeRecoveryOperationReceipt {
        &self.receipt
    }
    pub const fn staged_media(
        &self,
    ) -> &worth_store_physical_backend::ClosedNonCurrentStagingMedia {
        self.receipt.backend.media()
    }
}

impl AuthorizedPointInTimeRecoveryPlan {
    pub fn ready<'a>(
        self,
        control: &'a OperationalControlStore,
        transition_id: OperationalTransitionId,
        current: &StoreCurrentAuthorityWitness,
        observed_at: u64,
        revocation: AuthorizationRevocationObservation,
    ) -> Result<ExecutionReadyPointInTimeRecovery<'a>, PitrReadinessDenial> {
        self.ready_through(
            control,
            control,
            transition_id,
            current,
            observed_at,
            revocation,
        )
    }

    #[cfg(any(test, feature = "certification-test-authority"))]
    pub fn ready_with_certification_control_store<'a>(
        self,
        control: &'a OperationalControlStore,
        append: &'a dyn crate::OperationalControlStorePort,
        transition_id: OperationalTransitionId,
        current: &StoreCurrentAuthorityWitness,
        observed_at: u64,
        revocation: AuthorizationRevocationObservation,
    ) -> Result<ExecutionReadyPointInTimeRecovery<'a>, PitrReadinessDenial> {
        self.ready_through(
            control,
            append,
            transition_id,
            current,
            observed_at,
            revocation,
        )
    }

    fn ready_through<'a>(
        self,
        control: &'a OperationalControlStore,
        append: &'a dyn crate::OperationalControlStorePort,
        transition_id: OperationalTransitionId,
        current: &StoreCurrentAuthorityWitness,
        observed_at: u64,
        revocation: AuthorizationRevocationObservation,
    ) -> Result<ExecutionReadyPointInTimeRecovery<'a>, PitrReadinessDenial> {
        if self.authorization.binding().authority_identity() != current.authority_identity() {
            return Err(PitrReadinessDenial::StaleAuthority);
        }
        let target_parent =
            self.backend
                .binding()
                .staging_root()
                .parent()
                .ok_or(PitrReadinessDenial::Target(
                    crate::control_store::NonCurrentRecoveryTargetDenial::Unavailable,
                ))?;
        let target_admission = control
            .admit_non_current_recovery_target(target_parent)
            .map_err(PitrReadinessDenial::Target)?;
        let operation_id = self.operation_id;
        let staging_authority = self.authorization.binding().authority_identity();
        let security_scope = self.authorization.binding().security_scope();
        let consumed = consume_authorization_through(
            control,
            append,
            operation_id.clone(),
            transition_id,
            self.authorization,
            Some(self.backend.binding().fingerprint()),
            observed_at,
            revocation,
        )
        .map_err(PitrReadinessDenial::Authorization)?;
        Ok(ExecutionReadyPointInTimeRecovery {
            operation_id,
            authorization: consumed.receipt(),
            staging_authority,
            security_scope,
            backend: self.backend,
            recovery: self.recovery,
            lease: self.lease,
            owner_verification: self.owner_verification,
            _target_admission: target_admission,
            control: append,
        })
    }
}

impl LoweredPointInTimeRecoveryPlan {
    pub fn recover_ready<'a>(
        self,
        handle: &crate::IndeterminateRecoveryStagingHandle,
        control: &'a OperationalControlStore,
        current: &StoreCurrentAuthorityWitness,
    ) -> Result<ExecutionReadyPointInTimeRecovery<'a>, PitrReadinessDenial> {
        let binding = self.authorization.binding();
        if handle.operation_kind() != crate::RecoveryStagingOperationKind::PointInTimeRecovery
            || self.operation_id != *handle.operation_id()
            || binding.authority_identity() != current.authority_identity()
            || handle.authority_identity() != current.authority_identity()
            || binding.fingerprint() != handle.plan_fingerprint()
            || self.backend.binding().fingerprint() != handle.execution_plan_fingerprint()
        {
            return Err(PitrReadinessDenial::StaleAuthority);
        }
        let target_parent =
            self.backend
                .binding()
                .staging_root()
                .parent()
                .ok_or(PitrReadinessDenial::Target(
                    crate::NonCurrentRecoveryTargetDenial::Unavailable,
                ))?;
        let target_admission = control
            .admit_non_current_recovery_target(target_parent)
            .map_err(PitrReadinessDenial::Target)?;
        let authorization = recover_authorization_consumption(
            control,
            handle.operation_id(),
            handle.authorization_identity(),
            handle.plan_fingerprint(),
        )
        .map_err(PitrReadinessDenial::Authorization)?;
        Ok(ExecutionReadyPointInTimeRecovery {
            operation_id: self.operation_id,
            authorization,
            staging_authority: binding.authority_identity(),
            security_scope: binding.security_scope(),
            backend: self.backend,
            recovery: self.recovery,
            lease: self.lease,
            owner_verification: self.owner_verification,
            _target_admission: target_admission,
            control,
        })
    }
}

#[cfg(any(test, feature = "certification-test-authority"))]
impl<'a> ExecutionReadyPointInTimeRecovery<'a> {
    pub fn with_certification_control_store(
        mut self,
        control: &'a dyn crate::OperationalControlStorePort,
    ) -> Self {
        self.control = control;
        self
    }
}

impl ExecutionReadyPointInTimeRecovery<'_> {
    pub fn execute<Ports>(
        self,
        ports: &Ports,
    ) -> Result<ExecutedPointInTimeRecovery, PitrExecutionDenial>
    where
        Ports:
            crate::StagingAuthorizationContinuationPort + crate::workflow::StagedWalApplicationPort,
    {
        let staging_authority = self.staging_authority;
        let security_scope = self.security_scope;
        let mut continuation = StagingAuthorizationContinuation::new(self.authorization, ports);
        let staged = PhysicalRecoveryStagingOwner::execute_lowered_guarded_with_owner_effect(
            self.backend,
            |boundary| continuation.admit(boundary),
            |staging| PointInTimeReplayOwner::execute(self.recovery, staging, ports),
        );
        let (backend, recovery) = match staged {
            Ok(receipts) => receipts,
            Err(NonCurrentStagingOwnerExecutionDenial::Backend(
                NonCurrentStagingExecutionDenial::ContinuationDenied { .. },
            )) => {
                return Err(PitrExecutionDenial::Authorization(
                    continuation
                        .denial()
                        .expect("a denied gate records its cause"),
                ))
            }
            Err(NonCurrentStagingOwnerExecutionDenial::Backend(denial)) => {
                return Err(PitrExecutionDenial::Backend(denial))
            }
            Err(NonCurrentStagingOwnerExecutionDenial::Owner(denial)) => {
                return Err(PitrExecutionDenial::Recovery(denial))
            }
        };
        persist_recovery_owner_receipts(
            self.control,
            staging_authority,
            &self.operation_id,
            self.authorization,
            crate::OperationalWorkflowKind::PointInTimeRecovery,
            &backend,
            crate::workflow::point_in_time_recovery::pitr_owner_receipt_identity(recovery),
        )
        .map_err(PitrExecutionDenial::Control)?;
        record_recovery_staging_completion(
            self.control,
            staging_authority,
            self.operation_id.clone(),
            self.authorization,
            backend.plan_fingerprint(),
            backend.media().content_fingerprint(),
        )
        .map_err(PitrExecutionDenial::Control)?;
        Ok(ExecutedPointInTimeRecovery {
            operation_id: self.operation_id,
            staging_authority,
            security_scope,
            source_lease: self.lease,
            owner_verification: self.owner_verification,
            receipt: PointInTimeRecoveryOperationReceipt {
                authorization: self.authorization,
                backend,
                recovery,
            },
        })
    }
}
