//! Fund semantic comparison outside guards, then retain its actual original custody.
use super::*;
use crate::domain_computation::primary_graph::request_execution::WorthQueryAdvancementPhase;

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

impl<S: worth_query_installation::facade::ApplicationSchema> NavigationAttempt<'_, S> {
    pub(super) fn readmit_output_profile(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        supplied: Option<&WorthQueryHistoryOutputProfile>,
    ) -> Result<ComparedOutputProfile, WorthQueryHistoryNavigationDenial> {
        use WorthQueryHistoryNavigationDenial as Denial;
        let (application, original) = {
            let progress = match self.progress.try_lock() {
                Ok(progress) => progress,
                Err(std::sync::TryLockError::WouldBlock) => return Err(Denial::Busy),
                Err(std::sync::TryLockError::Poisoned(poison)) => poison.into_inner(),
            };
            (progress.application, progress.output_profile.clone())
        };
        // The full canonical comparison can allocate and walk variable input.
        // Neither an attempt nor a document-selection guard remains held.
        let equal = match (original.as_ref(), supplied) {
            (None, None) => true,
            (Some(original), Some(supplied)) => {
                let execution = phase
                    .checked_execution_request_for(&application.runtime.product_runtime)
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
