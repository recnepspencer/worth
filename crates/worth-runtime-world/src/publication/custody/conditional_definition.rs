use std::sync::{Arc, Mutex};

use worth_runtime_bridge::facade::BridgeAppliedConditionalInstallationExtension;

use super::ActiveAttemptCustody;

/// Allocated before owner execution and retained by the attempt's existing
/// catalog record. Its caller lease never holds a World lock across Bridge work.
#[derive(Default)]
pub(crate) struct ConditionalDefinitionAttemptCustody {
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
        self.applied
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
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
