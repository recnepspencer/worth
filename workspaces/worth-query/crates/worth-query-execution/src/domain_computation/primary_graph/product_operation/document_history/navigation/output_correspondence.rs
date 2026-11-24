//! Fund semantic comparison outside guards, then retain its actual original custody.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;

pub(super) struct ComparedOutputProfile {
    original: Option<WorthQueryHistoryOutputProfile>,
}

impl ComparedOutputProfile {
    pub(super) fn matches_current(&self, current: Option<&WorthQueryHistoryOutputProfile>) -> bool {
        match (self.original.as_ref(), current) {
            (None, None) => true,
            (Some(original), Some(current)) => original.same_custody(current),
            _ => false,
        }
    }
}

impl NavigationAttempt {
    pub(super) fn readmit_output_profile<S: worth_query_installation::facade::ApplicationSchema>(
        &self,
        application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>,
        phase: &WorthQueryAdvancementPhase<'_>,
        supplied: Option<&WorthQueryHistoryOutputProfile>,
    ) -> Result<ComparedOutputProfile, WorthQueryHistoryNavigationDenial> {
        use WorthQueryHistoryNavigationDenial as Denial;
        let original = {
            let progress = match self.progress.try_lock() {
                Ok(progress) => progress,
                Err(std::sync::TryLockError::WouldBlock) => return Err(Denial::Busy),
                Err(std::sync::TryLockError::Poisoned(poison)) => poison.into_inner(),
            };
            progress.output_profile.clone()
        };
        // The full canonical comparison can allocate and walk variable input.
        // Neither an attempt nor a document-selection guard remains held.
        let equal = match (original.as_ref(), supplied) {
            (None, None) => true,
            (Some(original), Some(supplied)) => {
                let execution = phase
                    .checked_execution_request_for(application.inner.runtime.product_runtime_ref())
                    .map_err(Denial::Stopped)?;
                original.equivalent(&execution, supplied).map_err(|cause| {
                    Denial::Document(super::super::WorthQueryDocumentHistoryDenial::Recording(
                        cause,
                    ))
                })?
            }
            _ => false,
        };
        if !equal {
            return Err(Denial::IntentDrift);
        }
        Ok(ComparedOutputProfile { original })
    }
}
