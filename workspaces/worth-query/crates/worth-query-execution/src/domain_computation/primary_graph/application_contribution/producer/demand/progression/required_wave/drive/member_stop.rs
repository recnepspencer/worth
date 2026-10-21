//! A checkpoint stop is recorded on its actual successor before leaving apply.
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry;

enum Recording {
    Member,
    Successor,
}

pub(super) struct MemberStop {
    pub index: usize,
    pub stop: WorthQueryOutputDemandDenial,
    recording: StopRecording,
}
impl MemberStop {
    pub(super) fn member(index: usize, stop: WorthQueryOutputDemandDenial) -> Self {
        Self {
            index,
            stop,
            recording: StopRecording::member(),
        }
    }
    pub(super) fn checkpoint(
        index: usize,
        registry: &WorthQueryOutputDemandRegistry,
        successor: &WorthQueryOutputDemandKey,
        stop: WorthQueryOutputDemandDenial,
    ) -> Self {
        registry.record_required_stop(successor, &stop);
        Self {
            index,
            stop,
            recording: StopRecording(Recording::Successor),
        }
    }
    pub(super) fn into_parts(self) -> (usize, WorthQueryOutputDemandDenial, StopRecording) {
        (self.index, self.stop, self.recording)
    }
}

pub(super) struct StopRecording(Recording);
impl StopRecording {
    /// The continuation owner already recorded this on its genuine successor.
    pub(super) fn successor() -> Self {
        Self(Recording::Successor)
    }

    pub(super) fn member() -> Self {
        Self(Recording::Member)
    }
    pub(super) fn record_if_needed(
        self,
        registry: &WorthQueryOutputDemandRegistry,
        member: &WorthQueryOutputDemandKey,
        stop: &WorthQueryOutputDemandDenial,
    ) {
        match self.0 {
            Recording::Member => registry.record_required_stop(member, stop),
            Recording::Successor => {}
        }
    }
}
