use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::facade::runtime::{
    CommittedObservationEventSummary, ObservationBoundaryOutcome, ObservationHandle,
    ObservationHandleId, ObservationListener, ObservationNotice, ObservationPolicy,
    ObservationReadContext, ObservationTrigger, ObservedNodeSet, ObserverId,
};
use crate::facade::{NodeId, SignalError};

use super::runtime::SignalApp;
use super::signal::Signal;

impl SignalApp {
    pub fn observe_nodes(
        &mut self,
        policy: ObservationPolicy,
        nodes: impl IntoIterator<Item = NodeId>,
        listener: Box<dyn ObservationListener<(), (), (), (), ()>>,
    ) -> ObservationHandle {
        self.observations
            .register_nodes(policy, ObservedNodeSet::from_nodes(nodes), listener)
    }

    pub fn observe<T: Clone + Send + Sync + 'static>(
        &mut self,
        signal: Signal<T>,
        policy: ObservationPolicy,
        listener: Box<dyn ObservationListener<(), (), (), (), ()>>,
    ) -> ObservationHandle {
        self.observe_nodes(policy, [signal.node], listener)
    }

    pub fn watch<T, F>(&mut self, signal: Signal<T>, on_change: F) -> ObservationHandle
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&ObservationNotice<'_>) + Send + Sync + 'static,
    {
        self.observe(
            signal,
            ObservationPolicy::meaningful_change(),
            Box::new(EasyWatchListener { on_change }),
        )
    }

    pub fn effect<T, F>(&mut self, signal: Signal<T>, effect: F) -> ObservationHandle
    where
        T: Clone + Send + Sync + 'static,
        F: Fn() + Send + Sync + 'static,
    {
        self.observe(
            signal,
            ObservationPolicy::meaningful_change(),
            Box::new(EasyEffectListener { effect }),
        )
    }

    pub fn unobserve(&mut self, handle: ObservationHandle) -> bool {
        self.observations.unsubscribe(handle)
    }
}

pub(super) fn deliver_observation_boundary(
    app: &mut SignalApp,
    changed_nodes: BTreeSet<NodeId>,
) -> Result<(), SignalError> {
    if changed_nodes.is_empty() || app.observations.registration_count() == 0 {
        return Ok(());
    }

    let mut impacted_nodes = collect_impacted_observed_nodes(app, &changed_nodes)?;
    if impacted_nodes.is_empty() {
        return Ok(());
    }

    let mut recomputed_nodes = BTreeSet::new();
    let mut meaningful_nodes = BTreeSet::new();
    let computed_impacted_nodes = impacted_nodes
        .iter()
        .copied()
        .filter(|node| app.computed.contains_key(node))
        .collect::<Vec<_>>();
    for node in computed_impacted_nodes {
        let computed_meaningful_nodes = app.ensure_evaluated(node)?;
        recomputed_nodes.insert(node);
        meaningful_nodes.extend(computed_meaningful_nodes);
    }

    impacted_nodes.extend(changed_nodes.iter().copied());
    let mut matched_by_observer = BTreeMap::<ObserverId, Vec<NodeId>>::new();
    for node in impacted_nodes.iter().copied() {
        app.observations
            .for_each_matching_observer_for_node(node, |observer_id| {
                matched_by_observer
                    .entry(observer_id)
                    .or_default()
                    .push(node);
            });
    }

    let mut boundary_events = Vec::new();
    for (observer_id, matched_nodes) in matched_by_observer {
        let registration: Option<(ObservationHandleId, ObservationPolicy, &ObservedNodeSet)> =
            app.observations.registration_for(observer_id);
        let Some((handle_id, policy, observed_nodes)) = registration else {
            continue;
        };
        if matched_nodes.is_empty() {
            continue;
        }

        let visited = true;
        let recomputed = matched_nodes
            .iter()
            .copied()
            .any(|node| recomputed_nodes.contains(&node));
        let meaningful_change = matched_nodes
            .iter()
            .copied()
            .any(|node| meaningful_nodes.contains(&node) || changed_nodes.contains(&node));
        let trigger_matched = match policy.trigger() {
            ObservationTrigger::Visited => visited,
            ObservationTrigger::Recomputed => recomputed,
            ObservationTrigger::MeaningfulChange => meaningful_change,
        };
        if !trigger_matched {
            continue;
        }

        boundary_events.push(CommittedObservationEventSummary {
            observer_id,
            handle_id,
            policy,
            observed_nodes: observed_nodes.clone(),
            matched_nodes: ObservedNodeSet::from_nodes(matched_nodes),
            visited,
            recomputed,
            meaningful_change,
            trigger_matched,
            outcome: ObservationBoundaryOutcome::Delivered,
        });
    }

    let _ = app
        .observations
        .deliver_boundary_summaries(&app.graph, &boundary_events);
    Ok(())
}

fn collect_impacted_observed_nodes(
    app: &SignalApp,
    changed_nodes: &BTreeSet<NodeId>,
) -> Result<BTreeSet<NodeId>, SignalError> {
    if app.observations.registration_count() == 0 {
        return Ok(BTreeSet::new());
    }

    let mut impacted = BTreeSet::new();
    let mut visited = changed_nodes.clone();
    let mut queue = changed_nodes.iter().copied().collect::<VecDeque<_>>();
    while let Some(node) = queue.pop_front() {
        if app.observations.has_matching_observers_for_node(node) {
            impacted.insert(node);
        }
        for subscriber in app.graph.subscribers_of(node)? {
            if visited.insert(*subscriber) {
                queue.push_back(*subscriber);
            }
        }
    }
    Ok(impacted)
}

struct EasyWatchListener<F> {
    on_change: F,
}

impl<F> ObservationListener<(), (), (), (), ()> for EasyWatchListener<F>
where
    F: Fn(&ObservationNotice<'_>) + Send + Sync + 'static,
{
    fn on_observation(
        &self,
        _ctx: ObservationReadContext<'_, (), (), (), (), ()>,
        notice: &ObservationNotice<'_>,
    ) {
        (self.on_change)(notice);
    }
}

struct EasyEffectListener<F> {
    effect: F,
}

impl<F> ObservationListener<(), (), (), (), ()> for EasyEffectListener<F>
where
    F: Fn() + Send + Sync + 'static,
{
    fn on_observation(
        &self,
        _ctx: ObservationReadContext<'_, (), (), (), (), ()>,
        _notice: &ObservationNotice<'_>,
    ) {
        (self.effect)();
    }
}
