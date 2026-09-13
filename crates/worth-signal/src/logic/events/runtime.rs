use std::marker::PhantomData;

use crate::clock::RuntimeInstant;
use crate::data::checkpoint::CheckpointBarrier;
use crate::data::event_subscriber::{EventSubscriber, SubscriberId};
use crate::data::subscriber_context::SubscriberContext;
use crate::data::telemetry::RuntimeTelemetry;

use super::errors::{EventFlushError, SubscriberRegistryError};
use super::ordering::resolve_order;

pub(super) struct SubscriberEntry<E, D, C>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
{
    pub(super) id: SubscriberId,
    pub(super) name: &'static str,
    pub(super) requires: &'static [D],
    pub(super) provides: &'static [D],
    pub(super) routed_event_keys: Option<&'static [u64]>,
    pub(super) sub: Box<dyn EventSubscriber<Event = E, DataId = D, RuntimeContext = C>>,
}

/// Typed deterministic event bus.
///
/// Events are buffered in-order and delivered at checkpoint flush.
pub struct EventBus<E, D, C = ()>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
{
    pending: Vec<E>,
    subscribers: Vec<SubscriberEntry<E, D, C>>,
    order: Vec<usize>,
    context: SubscriberContext<D>,
    telemetry: RuntimeTelemetry,
    capture_telemetry: bool,
    finalized: bool,
    rollback_ready: Vec<bool>,
    begin_ready: Vec<bool>,
    context_marker: PhantomData<fn(&mut C)>,
    event_router: Option<Box<dyn Fn(&E) -> u64 + Send + Sync>>,
}

impl<E, D, C> Default for EventBus<E, D, C>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<E, D, C> EventBus<E, D, C>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
{
    /// Create an empty bus.
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            subscribers: Vec::new(),
            order: Vec::new(),
            context: SubscriberContext::new(),
            telemetry: RuntimeTelemetry::default(),
            capture_telemetry: true,
            finalized: false,
            rollback_ready: Vec::new(),
            begin_ready: Vec::new(),
            context_marker: PhantomData,
            event_router: None,
        }
    }

    /// Access committed subscriber context.
    pub fn context(&self) -> &SubscriberContext<D> {
        &self.context
    }

    /// Access committed subscriber context mutably.
    pub fn context_mut(&mut self) -> &mut SubscriberContext<D> {
        &mut self.context
    }

    /// Runtime telemetry snapshot.
    pub fn telemetry(&self) -> &RuntimeTelemetry {
        &self.telemetry
    }

    /// Reset runtime telemetry counters.
    pub fn reset_telemetry(&mut self) {
        self.telemetry = RuntimeTelemetry::default();
    }

    pub(crate) fn set_telemetry_capture(&mut self, capture: bool) {
        self.capture_telemetry = capture;
    }

    /// Register one subscriber (must call `finalize_registration` before use).
    pub fn subscribe(
        &mut self,
        sub: Box<dyn EventSubscriber<Event = E, DataId = D, RuntimeContext = C>>,
    ) -> Result<(), SubscriberRegistryError<D>> {
        self.subscribe_internal(None, sub)
    }

    pub fn subscribe_routed(
        &mut self,
        routed_event_keys: &'static [u64],
        sub: Box<dyn EventSubscriber<Event = E, DataId = D, RuntimeContext = C>>,
    ) -> Result<(), SubscriberRegistryError<D>> {
        self.subscribe_internal(Some(routed_event_keys), sub)
    }

    pub fn set_event_router<F>(&mut self, router: F)
    where
        F: Fn(&E) -> u64 + Send + Sync + 'static,
    {
        self.event_router = Some(Box::new(router));
    }

    fn subscribe_internal(
        &mut self,
        routed_event_keys: Option<&'static [u64]>,
        sub: Box<dyn EventSubscriber<Event = E, DataId = D, RuntimeContext = C>>,
    ) -> Result<(), SubscriberRegistryError<D>> {
        let id = sub.id();
        let name = sub.name();
        let requires = sub.requires();
        let provides = sub.provides();

        if let Some(existing) = self.subscribers.iter().find(|e| e.id == id) {
            return Err(SubscriberRegistryError::DuplicateSubscriberId {
                id,
                first: existing.name,
                second: name,
            });
        }
        self.subscribers.push(SubscriberEntry {
            id,
            name,
            requires,
            provides,
            routed_event_keys,
            sub,
        });
        self.rollback_ready.push(false);
        self.begin_ready.push(false);
        self.finalized = false;
        Ok(())
    }

    /// Validate dependency DAG and compute deterministic order.
    pub fn finalize_registration(&mut self) -> Result<(), SubscriberRegistryError<D>> {
        self.order = resolve_order(&self.subscribers)?;
        self.finalized = true;
        Ok(())
    }

    fn ensure_finalized(&mut self) -> Result<(), SubscriberRegistryError<D>> {
        if self.finalized {
            return Ok(());
        }
        self.finalize_registration()
    }

    /// Signal start of an operation lifecycle.
    pub fn begin(&mut self, runtime: &mut C) -> Result<(), SubscriberRegistryError<D>> {
        self.ensure_finalized()?;
        self.context.clear_staged();
        self.rollback_ready.clear();
        self.rollback_ready.resize(self.subscribers.len(), false);
        self.begin_ready.clear();
        self.begin_ready.resize(self.subscribers.len(), false);
        for &idx in &self.order {
            self.subscribers[idx]
                .sub
                .on_begin(&mut self.context, runtime);
            if let Some(flag) = self.begin_ready.get_mut(idx) {
                *flag = true;
            }
        }
        Ok(())
    }

    /// Record one event for later checkpoint delivery.
    pub fn emit(&mut self, event: E) {
        self.pending.push(event);
    }

    /// Flush pending events and run checkpoint lifecycle in deterministic order.
    pub fn flush(
        &mut self,
        barrier: CheckpointBarrier,
        runtime: &mut C,
    ) -> Result<Vec<CompletedSubscriber>, EventFlushError<D>> {
        self.flush_with_capture(barrier, runtime, self.capture_telemetry)
    }

    pub(crate) fn flush_with_capture(
        &mut self,
        barrier: CheckpointBarrier,
        runtime: &mut C,
        capture_telemetry: bool,
    ) -> Result<Vec<CompletedSubscriber>, EventFlushError<D>> {
        let flush_start = RuntimeInstant::now();
        self.ensure_finalized().map_err(EventFlushError::Registry)?;

        let routed_batches = self.event_router.as_ref().map(|router| {
            let mut by_key = std::collections::HashMap::<u64, Vec<&E>>::new();
            for event in &self.pending {
                by_key.entry(router(event)).or_default().push(event);
            }
            by_key
        });
        let mut completed_subscribers = Vec::new();

        for &idx in &self.order {
            let entry = &mut self.subscribers[idx];
            let staged_before = self.context.staged_ids();
            if let Some(keys) = entry.routed_event_keys {
                if let Some(batches) = &routed_batches {
                    for key in keys {
                        if let Some(events) = batches.get(key) {
                            for event in events {
                                entry.sub.on_event(event);
                            }
                        }
                    }
                } else {
                    for event in &self.pending {
                        entry.sub.on_event(event);
                    }
                }
            } else {
                for event in &self.pending {
                    entry.sub.on_event(event);
                }
            }
            if let Err(source) = entry.sub.on_checkpoint(barrier, &mut self.context, runtime) {
                let staged_after = self.context.staged_ids();
                self.context.clear_staged();
                return Err(EventFlushError::Subscriber {
                    subscriber_id: entry.id,
                    subscriber_name: entry.name,
                    completed_subscribers,
                    failed_subscriber_requires: format_data_ids(entry.requires),
                    failed_subscriber_provides: format_data_ids(entry.provides),
                    failed_subscriber_staged: staged_delta(&staged_before, &staged_after),
                    source,
                });
            }
            if let Some(flag) = self.rollback_ready.get_mut(idx) {
                *flag = true;
            }
            completed_subscribers.push(CompletedSubscriber {
                name: entry.name,
                requires_data_ids: format_data_ids(entry.requires),
                provides_data_ids: format_data_ids(entry.provides),
                staged_data_ids: staged_delta(&staged_before, &self.context.staged_ids()),
            });
        }

        self.pending.clear();
        self.context.finalize();
        if capture_telemetry {
            self.telemetry.checkpoint.event_flushes += 1;
            self.telemetry.checkpoint.event_flush_nanos += flush_start.elapsed().as_nanos();
        }
        Ok(completed_subscribers)
    }

    /// Roll back lifecycle state in reverse deterministic order.
    pub fn rollback(&mut self, runtime: &mut C) {
        self.rollback_with_capture(runtime, self.capture_telemetry);
    }

    pub(crate) fn rollback_with_capture(&mut self, runtime: &mut C, capture_telemetry: bool) {
        self.pending.clear();
        self.context.clear_staged();
        for &idx in self.order.iter().rev() {
            if self.begin_ready.get(idx).copied().unwrap_or(false)
                || self.rollback_ready.get(idx).copied().unwrap_or(false)
            {
                self.subscribers[idx].sub.on_rollback(runtime);
            }
        }
        self.rollback_ready.fill(false);
        self.begin_ready.fill(false);
        if capture_telemetry {
            self.telemetry.checkpoint.rollback_count += 1;
        }
    }

    /// Deterministic resolved order for diagnostics/tests.
    pub fn resolved_order(&self) -> Vec<SubscriberId> {
        self.order
            .iter()
            .map(|&idx| self.subscribers[idx].id)
            .collect()
    }

    pub fn resolved_subscriber_names(&self) -> Vec<&'static str> {
        self.order
            .iter()
            .map(|&idx| self.subscribers[idx].name)
            .collect()
    }

    pub(crate) fn independent_branch_service_compatible(&self) -> bool {
        self.pending.is_empty()
            && self.subscribers.is_empty()
            && self.context.staged_ids().is_empty()
            && self.context.committed_ids().is_empty()
            && self.event_router.is_none()
    }
}

#[derive(Debug, Clone)]
pub struct CompletedSubscriber {
    pub name: &'static str,
    pub requires_data_ids: Vec<String>,
    pub provides_data_ids: Vec<String>,
    pub staged_data_ids: Vec<String>,
}

fn format_data_ids<D: std::fmt::Debug>(ids: &[D]) -> Vec<String> {
    ids.iter().map(|id| format!("{id:?}")).collect()
}

fn staged_delta<D>(before: &[D], after: &[D]) -> Vec<String>
where
    D: Copy + Ord + std::fmt::Debug,
{
    let before = before
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    after
        .iter()
        .filter(|id| !before.contains(id))
        .map(|id| format!("{id:?}"))
        .collect()
}
