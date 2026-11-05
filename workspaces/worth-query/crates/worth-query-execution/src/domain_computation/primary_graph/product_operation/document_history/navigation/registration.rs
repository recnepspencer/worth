//! One document-scoped attempt index, funded before candidate effects.
use super::super::{
    allocation::{reserve_arc, HistoryInsertionAllocation},
    WorthQueryDocumentHistory,
};
use super::*;
use std::{collections::BTreeMap, sync::Mutex};
use worth_query_declaration::facade::application_history::ApplicationHistoryBinding;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledDocumentHistory};
use {crate::domain_computation::request_execution::WorthQueryAdvancementPhase, crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding};

pub(in crate::domain_computation::primary_graph::product_operation::document_history) struct NavigationJournal<
    'runtime,
    S,
> {
    pub(super) attempts: BTreeMap<[u8; 32], RegisteredAttempt<'runtime, S>>,
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) capacity:
        Arc<super::pending_capacity::PendingNavigationCapacity>,
}

pub(super) struct RegisteredAttempt<'runtime, S> {
    pub(super) attempt: Arc<NavigationAttempt<'runtime, S>>,
    _allocation: HistoryInsertionAllocation,
}

impl<S> NavigationJournal<'_, S> {
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn new(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        resources: worth_query_declaration::facade::application_history::ApplicationHistoryNavigationResources,
    ) -> Result<Self, super::WorthQueryDocumentHistoryDenial> {
        Ok(Self {
            attempts: BTreeMap::new(),
            capacity: super::pending_capacity::PendingNavigationCapacity::new(
                execution,
                resources.maximum_unfinished_attempts,
            )?,
        })
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
    intent: WorthQueryHistoryNavigationIntent,
    meaning: WorthQueryApplicationIdempotencyBinding,
    output_profile: Option<WorthQueryHistoryOutputProfile>,
) -> Result<WorthQueryDocumentHistoryNavigation<'runtime, S>, WorthQueryHistoryNavigationDenial>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
{
    use WorthQueryHistoryNavigationDenial as Denial;
    let key = meaning.canonical_key();
    let (requested, source) = {
        let selected = document.selection.lock();
        if let Some(attempt) = selected
            .navigation
            .attempts
            .get(&key)
            .map(|entry| Arc::clone(&entry.attempt))
        {
            drop(selected);
            return bind(
                phase,
                document,
                attempt,
                &expected,
                intent,
                meaning,
                output_profile.as_ref(),
            );
        }
        if selected.expected != expected.physical
            || selected.journal.current_step() != &expected.step
        {
            return Err(Denial::HeadChanged);
        }
        selected.navigation.capacity.check()?;
        let requested = selected.journal.resolve_navigation(intent)?;
        selected
            .journal
            .prepare_navigation(requested)
            .map_err(Denial::Document)?;
        let source = selected
            .journal
            .preview(requested, document.model.clone())
            .map_err(Denial::Document)?;
        (requested, source)
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
        .checked_execution_request_for(&application.runtime.product_runtime)
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
            intent,
            meaning,
            output_profile.as_ref(),
        );
    }
    if selected.expected != expected.physical || selected.journal.current_step() != &expected.step {
        return Err(Denial::HeadChanged);
    }
    let pending_slot = selected
        .navigation
        .capacity
        .acquire(source.step.endpoint().product_branch())?;
    selected
        .journal
        .prepare_navigation(requested)
        .map_err(Denial::Document)?;
    let (registration_allocation, _) = HistoryInsertionAllocation::reserve::<(
        [u8; 32],
        RegisteredAttempt<'runtime, S>,
    )>(&execution, selected.navigation.attempts.len(), 0)
    .map_err(|cause| {
        Denial::Document(super::super::WorthQueryDocumentHistoryDenial::Recording(
            cause,
        ))
    })?;
    let head_slot = selected
        .journal
        .reserve_head(&execution)
        .map_err(Denial::Document)?;
    let attempt = Arc::new(NavigationAttempt {
        progress: Mutex::new(NavigationProgress {
            pending_slot: Some(pending_slot),
            application,
            meaning,
            output_profile,
            output_source: None,
            output_source_memory,
            expected,
            model: document.model.clone(),
            source: Some(source),
            requested,
            intent,
            phase: NavigationPhase::Prepared,
            head_slot: Some(head_slot),
            prior_cleanup: None,
            selection_pause: None,
            delivery_pause: None,
        }),
        _allocation: allocation,
    });
    selected.navigation.attempts.insert(
        key,
        RegisteredAttempt {
            attempt: Arc::clone(&attempt),
            _allocation: registration_allocation,
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
    intent: WorthQueryHistoryNavigationIntent,
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
        || progress.intent != intent
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
