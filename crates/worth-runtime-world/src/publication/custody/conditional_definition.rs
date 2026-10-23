use std::sync::{Arc, Mutex};

use worth_runtime_bridge::facade::BridgeAppliedConditionalInstallationExtension;
use worth_runtime_bridge::facade::BridgePreparedConditionalInstallationExtension;
use worth_signal::facade::branch::{
    AdmittedSignalConditionalDefinitionPublication, SignalConditionalDefinitionAdvanceBinding,
};

use super::ActiveAttemptCustody;

/// Allocated before owner execution and retained by the attempt's existing
/// catalog record. Its caller lease never holds a World lock across Bridge work.
#[derive(Default)]
pub(crate) struct ConditionalDefinitionAttemptCustody {
    prepared: Mutex<Option<BridgePreparedConditionalInstallationExtension>>,
    publication: Mutex<Option<AdmittedSignalConditionalDefinitionPublication>>,
    binding: Mutex<Option<SignalConditionalDefinitionAdvanceBinding>>,
    applied: Mutex<Option<BridgeAppliedConditionalInstallationExtension>>,
}

impl std::fmt::Debug for ConditionalDefinitionAttemptCustody {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ConditionalDefinitionAttemptCustody")
            .field("retained", &self.retains_definition())
            .finish()
    }
}

impl ConditionalDefinitionAttemptCustody {
    pub(crate) fn can_resume(&self, signal_performed: bool) -> bool {
        if signal_performed {
            self.applied
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_some()
        } else {
            self.prepared
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_some()
                && self
                    .publication
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .is_some()
        }
    }
    pub(crate) fn has_binding(&self) -> bool {
        self.binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
    }

    pub(crate) fn complete_activation(
        &self,
        bridge: &worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
    ) -> Result<(), worth_signal::facade::SignalError> {
        let mut applied = self.lease_applied();
        let binding = self
            .take_binding()
            .expect("completed Signal movement carries original activation binding");
        bridge
            .complete_owned_conditional_installation_activation(applied.applied_mut(), binding)
            .map_err(|(binding, denial)| {
                self.retain_binding(binding);
                worth_signal::facade::SignalError::invalid_input(format!("{denial:?}"))
            })
    }
    pub(crate) fn admit_bridge(
        &self,
        bridge: &worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
    ) -> Result<(), crate::recovery::RuntimeWorldRecoveryDenial> {
        let prepared = self
            .prepared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(prepared) = prepared {
            let lease = ConditionalDefinitionPreparationLease {
                custody: self,
                prepared: Some(prepared),
            };
            return bridge
                .admit_prepared_conditional_installation(
                    lease.prepared.as_ref().expect("prepared lease"),
                )
                .map_err(|_| {
                    crate::recovery::RuntimeWorldRecoveryDenial::SettlementEvidenceUnavailable
                });
        }
        let applied = self
            .applied
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        let applied = applied
            .ok_or(crate::recovery::RuntimeWorldRecoveryDenial::SettlementEvidenceUnavailable)?;
        let lease = ConditionalDefinitionApplicationLease {
            custody: self,
            applied: Some(applied),
        };
        bridge
            .admit_applied_conditional_installation(lease.applied.as_ref().expect("applied lease"))
            .map_err(|_| crate::recovery::RuntimeWorldRecoveryDenial::SettlementEvidenceUnavailable)
    }
    pub(crate) fn retain_prepared(&self, prepared: BridgePreparedConditionalInstallationExtension) {
        let mut slot = self
            .prepared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(slot.is_none());
        *slot = Some(prepared);
    }

    pub(crate) fn take_prepared(&self) -> BridgePreparedConditionalInstallationExtension {
        self.prepared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .expect("original prepared Bridge extension")
    }

    pub(crate) fn retain_publication(
        &self,
        publication: AdmittedSignalConditionalDefinitionPublication,
    ) {
        let mut slot = self
            .publication
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(slot.is_none());
        *slot = Some(publication);
    }

    pub(crate) fn take_publication(&self) -> AdmittedSignalConditionalDefinitionPublication {
        self.publication
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .expect("original admitted Signal publication")
    }

    pub(crate) fn retain_binding(&self, binding: SignalConditionalDefinitionAdvanceBinding) {
        let mut slot = self
            .binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(slot.is_none());
        *slot = Some(binding);
    }

    pub(crate) fn take_binding(&self) -> Option<SignalConditionalDefinitionAdvanceBinding> {
        self.binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }
    pub(crate) fn retain_applied(&self, applied: BridgeAppliedConditionalInstallationExtension) {
        let mut slot = self
            .applied
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(
            slot.is_none(),
            "one attempt applies one conditional definition"
        );
        *slot = Some(applied);
    }

    pub(crate) fn lease_applied(&self) -> ConditionalDefinitionApplicationLease<'_> {
        ConditionalDefinitionApplicationLease {
            custody: self,
            applied: Some(self.take_applied()),
        }
    }

    pub(crate) fn take_applied(&self) -> BridgeAppliedConditionalInstallationExtension {
        self.applied
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .expect("performed Signal publication carries Bridge completion")
    }

    pub(crate) fn retains_definition(&self) -> bool {
        self.prepared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
            || self
                .applied
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_some()
    }
}

struct ConditionalDefinitionPreparationLease<'a> {
    custody: &'a ConditionalDefinitionAttemptCustody,
    prepared: Option<BridgePreparedConditionalInstallationExtension>,
}

impl Drop for ConditionalDefinitionPreparationLease<'_> {
    fn drop(&mut self) {
        self.custody.retain_prepared(
            self.prepared
                .take()
                .expect("prepared lease restores exactly once"),
        );
    }
}

pub(crate) struct ConditionalDefinitionApplicationLease<'a> {
    custody: &'a ConditionalDefinitionAttemptCustody,
    applied: Option<BridgeAppliedConditionalInstallationExtension>,
}

impl ConditionalDefinitionApplicationLease<'_> {
    pub(crate) fn applied_mut(&mut self) -> &mut BridgeAppliedConditionalInstallationExtension {
        self.applied
            .as_mut()
            .expect("the application lease owns its candidate")
    }
}

impl Drop for ConditionalDefinitionApplicationLease<'_> {
    fn drop(&mut self) {
        self.custody.retain_applied(
            self.applied
                .take()
                .expect("the application lease restores exactly once"),
        );
    }
}

impl ActiveAttemptCustody {
    pub(crate) fn retain_conditional_definition_custody(
        &mut self,
        custody: Arc<ConditionalDefinitionAttemptCustody>,
    ) {
        let mut lease = self.lease_resources();
        let slot = &mut lease.resources_mut().conditional_definition;
        assert!(slot.is_none());
        *slot = Some(custody);
    }
    pub(crate) fn conditional_definition_custody(
        &mut self,
    ) -> Option<Arc<ConditionalDefinitionAttemptCustody>> {
        self.lease_resources()
            .resources_mut()
            .conditional_definition
            .clone()
    }
    pub(crate) fn reserve_conditional_definition_custody(
        &mut self,
    ) -> Arc<ConditionalDefinitionAttemptCustody> {
        let custody = Arc::new(ConditionalDefinitionAttemptCustody::default());
        let mut lease = self.lease_resources();
        let slot = &mut lease.resources_mut().conditional_definition;
        assert!(
            slot.is_none(),
            "one attempt reserves one conditional definition"
        );
        *slot = Some(Arc::clone(&custody));
        custody
    }
}
