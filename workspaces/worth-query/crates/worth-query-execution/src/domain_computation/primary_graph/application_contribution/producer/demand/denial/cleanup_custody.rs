//! Unique native cleanup transfer; diagnostic copies never contain authority.
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationCleanupRequired as Required,
    WorthQueryApplicationUncommitted,
};
use crate::domain_computation::WorthQueryDirectRunCleanupReceipt as Receipt;
// Erasure bounds auto-trait traversal through the retained native World graph.
trait CleanupOwner: std::fmt::Debug + Send + Sync {
    fn failure(&mut self) -> Option<&crate::domain_computation::WorthQueryMutationCleanupFailure>;
    fn into_required(self: Box<Self>) -> Result<Box<Required>, Box<dyn CleanupOwner>>;
    fn release(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<Option<Receipt>, super::WorthQueryOutputDemandDenial>;
}
#[derive(Debug)]
struct NativeCleanupState {
    required: Option<Box<Required>>,
    // Physical release never erases unpublished/uncertain producer evidence.
    released_outcome: Option<WorthQueryApplicationUncommitted>,
}
#[derive(Debug)]
struct NativeCleanupOwner(std::sync::Mutex<NativeCleanupState>);
impl CleanupOwner for NativeCleanupOwner {
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
                state.released_outcome = Some(outcome);
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
pub(super) struct DemandCleanupCustody(Box<dyn CleanupOwner>);
impl DemandCleanupCustody {
    pub(super) fn failure(
        &mut self,
    ) -> Option<&crate::domain_computation::WorthQueryMutationCleanupFailure> {
        self.0.failure()
    }

    pub(super) fn new(owner: Box<Required>) -> Self {
        Self(Box::new(NativeCleanupOwner(std::sync::Mutex::new(
            NativeCleanupState {
                required: Some(owner),
                released_outcome: None,
            },
        ))))
    }
    pub(super) fn into_required(self) -> Result<Box<Required>, Self> {
        self.0.into_required().map_err(Self)
    }
    pub(super) fn release(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<Option<Receipt>, super::WorthQueryOutputDemandDenial> {
        self.0.release(phase)
    }
}
