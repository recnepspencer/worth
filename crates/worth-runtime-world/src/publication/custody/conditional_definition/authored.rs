//! Actual Bridge preparation and installed completion belong to the original World attempt.
use super::*;
use worth_runtime_bridge::facade::{
    BridgeDurableDefinitionCompletion, BridgeDurableDefinitionCompletionDenial,
    BridgePreparedAuthoredDefinitionPublication,
};

#[derive(Debug)]
pub(crate) struct AuthoredConditionalDefinitionAttemptCustody {
    prepared: Mutex<Option<BridgePreparedAuthoredDefinitionPublication>>,
    completion: BridgeDurableDefinitionCompletion,
}
impl AuthoredConditionalDefinitionAttemptCustody {
    pub(crate) const fn metadata_charge_hint() -> usize {
        std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>()
    }
    pub(crate) fn new(prepared: BridgePreparedAuthoredDefinitionPublication) -> Arc<Self> {
        let completion = prepared.completion();
        Arc::new(Self {
            prepared: Mutex::new(Some(prepared)),
            completion,
        })
    }
    pub(crate) fn take_prepared(&self) -> BridgePreparedAuthoredDefinitionPublication {
        self.prepared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .expect("one original consumes its actual Bridge preparation")
    }
    pub(crate) fn complete(
        &self,
        output: &worth_signal::facade::durability::SignalDurableAdvanceOutput,
    ) -> Result<(), BridgeDurableDefinitionCompletionDenial> {
        self.completion.installed_lowering(output).map(|_| ())
    }
}
impl ConditionalDefinitionAttemptCustody {
    pub(crate) fn retain_authored(
        &self,
        original: Arc<AuthoredConditionalDefinitionAttemptCustody>,
    ) {
        let mut slot = self
            .authored
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(slot.is_none());
        *slot = Some(original);
    }
}
