use super::super::runtime_world::{Domain, Ev, Impact, Tier};
use crate::facade::{
    DiagnosticsTier, ObservationListener, ObservationNotice, ObservationReadContext,
};
use std::sync::{Arc, Mutex};

pub(in crate::logic::transaction::tests) struct NoopObservationListener;

impl ObservationListener<Domain, Impact, Ev, (), Tier> for NoopObservationListener {
    fn on_observation(
        &self,
        _ctx: ObservationReadContext<'_, Domain, Impact, Ev, (), Tier>,
        _notice: &ObservationNotice<'_>,
    ) {
    }
}

pub(in crate::logic::transaction::tests) struct RecordingObservationListener {
    pub(in crate::logic::transaction::tests) calls: Arc<Mutex<Vec<(u64, DiagnosticsTier)>>>,
}

impl ObservationListener<Domain, Impact, Ev, (), Tier> for RecordingObservationListener {
    fn on_observation(
        &self,
        ctx: ObservationReadContext<'_, Domain, Impact, Ev, (), Tier>,
        notice: &ObservationNotice<'_>,
    ) {
        let branch = ctx.current_branch();
        let profile = ctx.diagnostics_profile();
        self.calls
            .lock()
            .expect("observation preview mutex poisoned")
            .push((branch.id.0, profile));
        assert_eq!(notice.observer_id().get(), 1);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::logic::transaction::tests) struct CommittedObservationRecord {
    pub(in crate::logic::transaction::tests) observer_id: u64,
    pub(in crate::logic::transaction::tests) handle_id: u64,
    pub(in crate::logic::transaction::tests) matched_node_count: usize,
    pub(in crate::logic::transaction::tests) visited: bool,
    pub(in crate::logic::transaction::tests) recomputed: bool,
    pub(in crate::logic::transaction::tests) meaningful_change: bool,
    pub(in crate::logic::transaction::tests) trigger_matched: bool,
}

pub(in crate::logic::transaction::tests) struct Phase3RecordingObservationListener {
    pub(in crate::logic::transaction::tests) calls: Arc<Mutex<Vec<CommittedObservationRecord>>>,
}

impl ObservationListener<Domain, Impact, Ev, (), Tier> for Phase3RecordingObservationListener {
    fn on_observation(
        &self,
        _ctx: ObservationReadContext<'_, Domain, Impact, Ev, (), Tier>,
        notice: &ObservationNotice<'_>,
    ) {
        self.calls
            .lock()
            .expect("phase3 observation mutex poisoned")
            .push(CommittedObservationRecord {
                observer_id: notice.observer_id().get(),
                handle_id: notice.handle_id().get(),
                matched_node_count: notice.matched_nodes().len(),
                visited: notice.visited(),
                recomputed: notice.recomputed(),
                meaningful_change: notice.meaningful_change(),
                trigger_matched: notice.trigger_matched(),
            });
    }
}
