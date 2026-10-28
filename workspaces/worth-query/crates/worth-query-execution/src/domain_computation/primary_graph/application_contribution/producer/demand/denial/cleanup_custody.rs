//! Unique native cleanup transfer; diagnostic copies never contain authority.
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationCleanupRequired as Required,
    WorthQueryApplicationUncommitted,
};
use crate::domain_computation::WorthQueryDirectRunCleanupReceipt as Receipt;
type Recovery = crate::domain_computation::WorthQueryProductUnpublishedRecovery;
type ReleaseFailure = crate::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure;
type ReleaseReceipt = crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt;
type ReadRelease<'a> = dyn FnMut(Recovery) -> Result<ReleaseReceipt, ReleaseFailure> + 'a;
// Erasure bounds auto-trait traversal through the retained native World graph.
trait CleanupOwner: std::fmt::Debug + Send + Sync {
    fn failure(&mut self) -> Option<&crate::domain_computation::WorthQueryMutationCleanupFailure>;
    fn into_required(self: Box<Self>) -> Result<Box<Required>, Box<dyn CleanupOwner>>;
    fn read_close(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        release: &mut ReadRelease<'_>,
    ) -> Result<(), super::super::WorthQueryOutputReadCloseDenial>;
    fn receipt(&mut self) -> Option<&Receipt>;
    fn outcome(&mut self) -> Option<&WorthQueryApplicationUncommitted>;
    fn retired_components(&mut self) -> usize;
    fn release(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<Option<Receipt>, super::WorthQueryOutputDemandDenial>;
}
#[derive(Debug)]
struct NativeCleanupState {
    required: Option<Box<Required>>,
    // Physical release never erases unpublished/uncertain producer evidence.
    outcome: Option<WorthQueryApplicationUncommitted>,
    released_receipt: Option<Receipt>,
    release_failure: Option<ReleaseFailure>,
    recovery_receipt: Option<ReleaseReceipt>,
}
#[derive(Debug)]
struct NativeCleanupOwner(std::sync::Mutex<NativeCleanupState>);
impl CleanupOwner for NativeCleanupOwner {
    fn read_close(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        release: &mut ReadRelease<'_>,
    ) -> Result<(), super::super::WorthQueryOutputReadCloseDenial> {
        use super::super::WorthQueryOutputReadCloseDenial as Denial;
        self.release(phase).map_err(Denial::Cleanup)?;
        let state = self
            .0
            .get_mut()
            .unwrap_or_else(|poison| poison.into_inner());
        use WorthQueryApplicationUncommitted as Outcome;
        let pending = if let Some(failure) = state.release_failure.take() {
            Some(match failure {
                ReleaseFailure::Recovery(failure) => release(failure.into_recovery()),
                ReleaseFailure::OwnerCleanup(failure) => failure
                    .into_cleanup()
                    .retry_in_advancement(phase)
                    .map_err(ReleaseFailure::OwnerCleanup),
            })
        } else if let Some(outcome) = state.outcome.take() {
            match outcome {
                // Move the ACTUAL retained record and release its effects loan.
                // No receipt or catalog handle reconstructs this continuation.
                Outcome::ProductUnpublished(unpublished) => {
                    Some(release(unpublished.into_recovery()))
                }
                other => {
                    state.outcome = Some(other);
                    None
                }
            }
        } else {
            None
        };
        if let Some(result) = pending {
            match result {
                Ok(receipt) => state.recovery_receipt = Some(receipt),
                Err(failure) => {
                    let denial = match &failure {
                        ReleaseFailure::Recovery(failure) => Denial::Recovery(failure.denial()),
                        ReleaseFailure::OwnerCleanup(failure) => Denial::Owner(failure.denial()),
                    };
                    state.release_failure = Some(failure);
                    return Err(denial);
                }
            }
        }
        use crate::domain_computation::WorthQueryManagedRunCleanupDisposition as Disposition;
        if state.released_receipt.as_ref().is_some_and(|receipt| {
            match receipt.inspection().disposition() {
                Disposition::CleanupComplete => false,
                Disposition::CleanupPending => true,
                Disposition::RecoveryRequired => state.recovery_receipt.is_none(),
            }
        }) || state.outcome.as_ref().is_some_and(|outcome| {
            matches!(
                outcome,
                Outcome::CleanupRequired(_) | Outcome::Deferred(_) | Outcome::Indeterminate(_)
            )
        }) {
            return Err(Denial::ProducerRecoveryRetained);
        }
        Ok(())
    }
    fn retired_components(&mut self) -> usize {
        self.0
            .get_mut()
            .unwrap_or_else(|poison| poison.into_inner())
            .recovery_receipt
            .as_ref()
            .map_or(0, ReleaseReceipt::retired_component_count)
    }
    fn outcome(&mut self) -> Option<&WorthQueryApplicationUncommitted> {
        let state = self
            .0
            .get_mut()
            .unwrap_or_else(|poison| poison.into_inner());
        state
            .required
            .as_ref()
            .map(|required| required.outcome())
            .or(state.outcome.as_ref())
    }
    fn receipt(&mut self) -> Option<&Receipt> {
        self.0
            .get_mut()
            .unwrap_or_else(|poison| poison.into_inner())
            .released_receipt
            .as_ref()
    }
    fn failure(&mut self) -> Option<&crate::domain_computation::WorthQueryMutationCleanupFailure> {
        self.0
            .get_mut()
            .unwrap_or_else(|poison| poison.into_inner())
            .required
            .as_mut()
            .map(|required| required.cleanup().failure())
    }

    fn into_required(mut self: Box<Self>) -> Result<Box<Required>, Box<dyn CleanupOwner>> {
        let state = self
            .0
            .get_mut()
            .unwrap_or_else(|poison| poison.into_inner());
        match state.required.take() {
            Some(required) => Ok(required),
            None => Err(self),
        }
    }
    fn release(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<Option<Receipt>, super::WorthQueryOutputDemandDenial> {
        let state = self
            .0
            .get_mut()
            .unwrap_or_else(|poison| poison.into_inner());
        let Some(required) = state.required.take() else {
            return Ok(None);
        };
        let (outcome, cleanup) = required.into_parts();
        match cleanup.retry_in_advancement(phase) {
            Ok(receipt) => {
                state.outcome = Some(outcome);
                state.released_receipt = Some(receipt.clone());
                Ok(Some(receipt))
            }
            Err(cleanup) => {
                let denial = cleanup
                    .last_retry_denial()
                    .cloned()
                    .map(super::WorthQueryOutputDemandDenial::request_admission)
                    .unwrap_or_else(|| {
                        super::WorthQueryOutputDemandDenial::new(
                            super::WorthQueryOutputDemandDenialKind::ManagedCleanupRequired(
                                cleanup.failure().kind(),
                            ),
                            "producer cleanup",
                        )
                    });
                state.required = Some(Required::from_parts(outcome, cleanup));
                Err(denial)
            }
        }
    }
}
#[derive(Debug)]
pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) struct DemandCleanupCustody(
    Box<dyn CleanupOwner>,
);
impl DemandCleanupCustody {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn retired_components(
        &mut self,
    ) -> usize {
        self.0.retired_components()
    }
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn from_outcome(
        outcome: WorthQueryApplicationUncommitted,
    ) -> Self {
        Self(Box::new(NativeCleanupOwner(std::sync::Mutex::new(
            NativeCleanupState {
                required: None,
                outcome: Some(outcome),
                released_receipt: None,
                release_failure: None,
                recovery_receipt: None,
            },
        ))))
    }
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn outcome(
        &mut self,
    ) -> Option<&WorthQueryApplicationUncommitted> {
        self.0.outcome()
    }
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn close_read(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        release: &mut ReadRelease<'_>,
    ) -> Result<(), super::super::WorthQueryOutputReadCloseDenial> {
        self.0.read_close(phase, release)
    }
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn receipt(
        &mut self,
    ) -> Option<&Receipt> {
        self.0.receipt()
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn failure(
        &mut self,
    ) -> Option<&crate::domain_computation::WorthQueryMutationCleanupFailure> {
        self.0.failure()
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn new(
        owner: Box<Required>,
    ) -> Self {
        Self(Box::new(NativeCleanupOwner(std::sync::Mutex::new(
            NativeCleanupState {
                required: Some(owner),
                outcome: None,
                released_receipt: None,
                release_failure: None,
                recovery_receipt: None,
            },
        ))))
    }
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn into_required(
        self,
    ) -> Result<Box<Required>, Self> {
        self.0.into_required().map_err(Self)
    }
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn release(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<Option<Receipt>, super::WorthQueryOutputDemandDenial> {
        self.0.release(phase)
    }
}
