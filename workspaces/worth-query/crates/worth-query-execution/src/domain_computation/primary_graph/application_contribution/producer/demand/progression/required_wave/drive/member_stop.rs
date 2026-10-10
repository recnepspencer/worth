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
            recording: StopRecording(Recording::Member),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_successor_checkpoint_delivery_failure_records_only_its_successor_row() {
        let (registry, predecessor, successor) =
            WorthQueryOutputDemandRegistry::checkpoint_stop_rows_for_test();
        let stop = WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::SchedulingRejected,
            "delivery has no successor",
        );
        let (_, stop, recording) =
            MemberStop::checkpoint(0, &registry, &successor, stop).into_parts();
        recording.record_if_needed(&registry, &predecessor, &stop);
        assert_eq!(
            stop.kind(),
            WorthQueryOutputDemandDenialKind::SchedulingRejected
        );
        assert_eq!(
            registry.recorded_stop_for_test(&successor),
            Some(WorthQueryOutputDemandDenialKind::SchedulingRejected),
            "the checkpoint's successor row owns the delivery failure"
        );
        assert_eq!(
            registry.recorded_stop_for_test(&predecessor),
            None,
            "the checkpoint's predecessor row carries no delivery failure"
        );
    }
}
