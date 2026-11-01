//! One document-scoped attempt index, funded before candidate effects.
use super::super::{
    allocation::{reserve_arc, HistoryInsertionAllocation},
    WorthQueryDocumentHistory, WorthQueryHistoryNodeId,
};
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationIdempotencyBinding,
};
use std::{collections::BTreeMap, sync::Mutex};
use worth_query_declaration::facade::application_history::ApplicationHistoryBinding;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledDocumentHistory};

pub(in crate::domain_computation::primary_graph::product_operation::document_history) struct NavigationJournal<
    'runtime,
    S,
> {
    pub(super) attempts: BTreeMap<[u8; 32], RegisteredAttempt<'runtime, S>>,
}

pub(super) struct RegisteredAttempt<'runtime, S> {
    pub(super) attempt: Arc<NavigationAttempt<'runtime, S>>,
    _allocation: HistoryInsertionAllocation,
}

impl<S> NavigationJournal<'_, S> {
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn new(
    ) -> Self {
        Self {
            attempts: BTreeMap::new(),
        }
    }
}

pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn prepare<
    'runtime,
    S,
    H,
>(
    phase: &WorthQueryAdvancementPhase<'_>,
    document: &WorthQueryDocumentHistory<'runtime, S, H>,
    installed: &WorthQueryInstalledDocumentHistory<S, H>,
    expected: WorthQueryHistorySelection,
    requested: WorthQueryHistoryNodeId,
    meaning: WorthQueryApplicationIdempotencyBinding,
    output_profile: Option<WorthQueryHistoryOutputProfile>,
) -> Result<WorthQueryDocumentHistoryNavigation<'runtime, S>, WorthQueryHistoryNavigationDenial>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
{
    use WorthQueryHistoryNavigationDenial as Denial;
    let key = meaning.canonical_key();
    let existing = document
        .selection
        .lock()
        .navigation
        .attempts
        .get(&key)
        .map(|entry| Arc::clone(&entry.attempt));
    if let Some(attempt) = existing {
        return bind(
            phase,
            document,
            attempt,
            &expected,
            requested,
            meaning,
            output_profile.as_ref(),
        );
    }
    let (source, view_after) = {
        let selected = document.selection.lock();
        if selected.expected != expected.physical
            || selected.journal.current_step() != &expected.step
        {
            return Err(Denial::HeadChanged);
        }
        let view_after = selected
            .journal
            .prepare_navigation(requested)
            .map_err(Denial::Document)?;
        let source = selected
            .journal
            .preview(requested, document.model.clone())
            .map_err(Denial::Document)?;
        (source, view_after)
    };
    let application = document.application();
    let current = application
        .on_branch(expected.physical.product_branch())
        .select(phase)
        .map_err(Denial::ProductAdmission)?;
    if !expected
        .physical
        .matches_observation(current.product().observation())
    {
        return Err(Denial::HeadChanged);
    }
    eligibility::require(
        phase,
        &current,
        installed,
        &document.model,
        &source,
        output_profile.as_ref(),
    )?;
    drop(current);
    let execution = phase
        .checked_execution_request_for(&application.product_runtime)
        .map_err(Denial::Stopped)?;
    let allocation =
        reserve_arc::<NavigationAttempt<'runtime, S>>(&execution).map_err(|cause| {
            Denial::Document(super::super::WorthQueryDocumentHistoryDenial::Recording(
                cause,
            ))
        })?;
    let output_source_memory = if output_profile.is_some() {
        reserve_arc::<crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation>(&execution)
            .map_err(|cause| Denial::Document(super::super::WorthQueryDocumentHistoryDenial::Recording(cause)))?
    } else {
        None
    };
    let attempt = Arc::new(NavigationAttempt {
        progress: Mutex::new(NavigationProgress {
            application,
            meaning,
            output_profile: output_profile.clone(),
            output_source: None,
            output_source_memory,
            expected: expected.clone(),
            model: document.model.clone(),
            source: Some(source),
            requested,
            view_after,
            phase: NavigationPhase::Prepared,
            previous: None,
            prior_cleanup: None,
            selection_pause: None,
            delivery_pause: None,
        }),
        _allocation: allocation,
    });
    let mut selected = document.selection.lock();
    if let Some(existing) = selected
        .navigation
        .attempts
        .get(&key)
        .map(|entry| Arc::clone(&entry.attempt))
    {
        drop(selected);
        return bind(
            phase,
            document,
            existing,
            &expected,
            requested,
            meaning,
            output_profile.as_ref(),
        );
    }
    if selected.expected != expected.physical || selected.journal.current_step() != &expected.step {
        return Err(Denial::HeadChanged);
    }
    let (allocation, _) = HistoryInsertionAllocation::reserve::<(
        [u8; 32],
        RegisteredAttempt<'runtime, S>,
    )>(&execution, selected.navigation.attempts.len(), 0)
    .map_err(|cause| {
        Denial::Document(super::super::WorthQueryDocumentHistoryDenial::Recording(
            cause,
        ))
    })?;
    selected.navigation.attempts.insert(
        key,
        RegisteredAttempt {
            attempt: Arc::clone(&attempt),
            _allocation: allocation,
        },
    );
    drop(selected);
    Ok(WorthQueryDocumentHistoryNavigation {
        cell: Arc::downgrade(&document.selection),
        attempt,
    })
}

fn bind<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>(
    phase: &WorthQueryAdvancementPhase<'_>,
    document: &WorthQueryDocumentHistory<'runtime, S, H>,
    attempt: Arc<NavigationAttempt<'runtime, S>>,
    expected: &WorthQueryHistorySelection,
    requested: WorthQueryHistoryNodeId,
    meaning: WorthQueryApplicationIdempotencyBinding,
    output_profile: Option<&WorthQueryHistoryOutputProfile>,
) -> Result<WorthQueryDocumentHistoryNavigation<'runtime, S>, WorthQueryHistoryNavigationDenial> {
    let compared = attempt.readmit_output_profile(phase, output_profile)?;
    let progress = match attempt.progress.try_lock() {
        Ok(progress) => progress,
        Err(std::sync::TryLockError::WouldBlock) => {
            return Err(WorthQueryHistoryNavigationDenial::Busy)
        }
        Err(std::sync::TryLockError::Poisoned(poison)) => poison.into_inner(),
    };
    if progress.expected != *expected
        || progress.requested != requested
        || !progress.meaning.has_same_intent(meaning)
        || !compared.matches_current(progress.output_profile.as_ref())
    {
        return Err(WorthQueryHistoryNavigationDenial::IntentDrift);
    }
    progress.require_resolvable()?;
    drop(progress);
    Ok(WorthQueryDocumentHistoryNavigation {
        cell: Arc::downgrade(&document.selection),
        attempt,
    })
}
