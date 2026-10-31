//! Final selection consumes the original current admission and exact performed D.
use super::super::WorthQueryDocumentHistory;
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryAdvancementPhase,
    WorthQueryApplicationIdempotencyBinding,
};
use worth_query_declaration::facade::application_history::ApplicationHistoryBinding;
use worth_query_installation::facade::ApplicationSchema;

impl<'runtime, S: ApplicationSchema> WorthQueryDocumentHistoryNavigation<'runtime, S> {
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn advance<
        H,
        O,
        I,
        M,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        admission: &WorthQueryAdmittedApplicationOperation<S, O, I, M>,
        meaning: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<Option<WorthQueryHistoryNavigationCompletion>, WorthQueryHistoryNavigationDenial>
    where
        H: ApplicationHistoryBinding<S>,
    {
        use WorthQueryHistoryNavigationDenial as Denial;
        let cell = self.cell.upgrade().ok_or(Denial::DocumentClosed)?;
        if !Arc::ptr_eq(&cell, &document.selection) {
            return Err(Denial::ForeignDocument);
        }
        let mut progress = match self.attempt.progress.try_lock() {
            Ok(progress) => progress,
            Err(std::sync::TryLockError::WouldBlock) => return Err(Denial::Busy),
            Err(std::sync::TryLockError::Poisoned(poison)) => poison.into_inner(),
        };
        if progress.meaning.canonical_key() != meaning.canonical_key()
            || !progress.meaning.has_same_intent(meaning)
        {
            return Err(Denial::IntentDrift);
        }
        if !progress.model.matches_operation(admission) {
            return Err(Denial::Document(
                super::super::WorthQueryDocumentHistoryDenial::ForeignModel,
            ));
        }
        admission
            .validate_current_authority()
            .map_err(Denial::Authorization)?;
        progress.require_resolvable()?;
        let completed = if let NavigationPhase::Selected(completed) = &progress.phase {
            completed.clone()
        } else {
            {
                let selected = cell.live().ok_or(Denial::DocumentClosed)?;
                if selected.expected != progress.expected.physical
                    || selected.journal.current_step() != &progress.expected.step
                {
                    return Err(Denial::HeadChanged);
                }
            }
            progress.retry_prior_cleanup(phase)?;
            if matches!(progress.phase, NavigationPhase::Selecting { .. }) {
                if let Some(pause) = &progress.selection_pause {
                    pause.pause();
                }
                let selected = cell.live().ok_or(Denial::DocumentClosed)?;
                if selected.expected != progress.expected.physical
                    || selected.journal.current_step() != &progress.expected.step
                {
                    return Err(Denial::HeadChanged);
                }
                admission
                    .validate_current_authority()
                    .map_err(Denial::Authorization)?;
                progress.select(phase, selected)?
            } else {
                progress.progress(phase, admission.publication_request())?;
                return Ok(None);
            }
        };
        // The cell guard has ended. A stop here retains Selected in the actual
        // registered core, so a later authorized retry resolves the original result.
        if let Some(pause) = &progress.delivery_pause {
            pause.pause();
        }
        admission
            .validate_current_authority()
            .map_err(Denial::Authorization)?;
        let execution = phase
            .checked_execution_request_for(&progress.application.product_runtime)
            .map_err(Denial::Stopped)?;
        execution.check_live().map_err(|cause| Denial::Stopped(crate::domain_computation::primary_graph::application_contribution::advancement_denial(worth_execution::WorkCeilingDenial::Stopped(cause))))?;
        Ok(Some(completed))
    }
}

impl<'runtime, S: ApplicationSchema> NavigationProgress<'runtime, S> {
    fn select(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        mut selected: super::super::selection::HistorySelectionGuard<'_, 'runtime, S>,
    ) -> Result<WorthQueryHistoryNavigationCompletion, WorthQueryHistoryNavigationDenial> {
        use WorthQueryHistoryNavigationDenial as Denial;
        // Every descriptor and custody destination exists before changing the head.
        let source = self
            .source
            .as_ref()
            .expect("pending navigation owns its retained source");
        let next_view = selected
            .journal
            .prepare_navigation(self.requested)
            .map_err(Denial::Document)?;
        if next_view != self.view_after {
            return Err(Denial::HeadChanged);
        }
        debug_assert!(self.previous.is_none());
        let NavigationPhase::Selecting {
            candidate,
            exact,
            _publication,
        } = std::mem::replace(&mut self.phase, NavigationPhase::Discarded)
        else {
            unreachable!()
        };
        let completion = WorthQueryHistoryNavigationCompletion {
            requested: self.requested,
            selection: WorthQueryHistorySelection {
                step: source.step.clone(),
                physical: crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                    exact.observation(),
                ),
            },
        };
        let returned = completion.clone();
        let physical = completion.selection.physical.clone();
        let (previous, release) = match selected.head.adopt_at_deferred(candidate, phase, &exact) {
            Ok(adopted) => adopted,
            Err(failure) => {
                drop(selected);
                let (candidate, cause) = failure.into_parts();
                self.phase = NavigationPhase::Selecting {
                    candidate,
                    exact,
                    _publication,
                };
                return Err(Denial::Adoption(cause));
            }
        };
        selected.expected = physical;
        let previous_source = std::mem::replace(&mut selected.physical_source, exact);
        selected
            .journal
            .complete_navigation(self.requested, self.view_after);
        self.previous = Some(previous);
        self.phase = NavigationPhase::Selected(completion);
        let source = self.source.take();
        drop(selected);
        drop(previous_source);
        drop(source);
        drop(_publication);
        drop(release);
        Ok(returned)
    }
}
