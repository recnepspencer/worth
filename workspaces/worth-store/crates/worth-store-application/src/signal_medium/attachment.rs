//! A failed attachment retains the original unshared physical runtime.
use worth_signal::facade::durability::SignalDurableHomeBinding;
use worth_signal::facade::durability::SignalDurableMediumDenial;
use worth_store::physical_runtime::{AdmittedRecordPlacementPolicy, ServingPhysicalRuntime};

pub struct StoreSignalMediumAttachmentRefused {
    cause: SignalDurableMediumDenial,
    serving: Box<ServingPhysicalRuntime>,
    binding: SignalDurableHomeBinding,
    checkpoint: Option<worth_signal::facade::durability::SignalAuthoredHomeImage>,
    placement: AdmittedRecordPlacementPolicy,
}
impl StoreSignalMediumAttachmentRefused {
    pub(super) fn new(
        cause: SignalDurableMediumDenial,
        serving: Box<ServingPhysicalRuntime>,
        binding: SignalDurableHomeBinding,
        checkpoint: Option<worth_signal::facade::durability::SignalAuthoredHomeImage>,
        placement: AdmittedRecordPlacementPolicy,
    ) -> Self {
        Self {
            cause,
            serving,
            binding,
            checkpoint,
            placement,
        }
    }
    pub fn cause(&self) -> &SignalDurableMediumDenial {
        &self.cause
    }
    /// Recover the SAME unshared allocation and image to correct an input
    /// mismatch. Retrying an unchanged stopped request uses `try_attach`.
    pub fn into_parts(
        self,
    ) -> (
        Box<ServingPhysicalRuntime>,
        SignalDurableHomeBinding,
        Option<worth_signal::facade::durability::SignalAuthoredHomeImage>,
        AdmittedRecordPlacementPolicy,
    ) {
        (self.serving, self.binding, self.checkpoint, self.placement)
    }
    pub fn try_attach(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            std::sync::Arc<ServingPhysicalRuntime>,
            std::sync::Arc<super::StoreSignalMedium>,
        ),
        Self,
    > {
        super::StoreSignalMedium::from_serving(
            self.serving,
            self.binding,
            self.checkpoint,
            self.placement,
            execution,
        )
    }
}
impl std::fmt::Debug for StoreSignalMediumAttachmentRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreSignalMediumAttachmentRefused")
            .field("cause", &self.cause)
            .finish_non_exhaustive()
    }
}
