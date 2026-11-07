use super::{
    growth::{compare, Axis},
    observation::*,
};
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};

struct InjectedFixture {
    phase: Phase,
    items: usize,
}
impl PhaseWorkSource for InjectedFixture {
    fn phase_work(&self) -> Vec<PhaseWork> {
        let host = SerialRequest::from_memory(
            SerialMemoryBudget::new(None),
            CancellationSource::new().token(),
            None,
        );
        let (_, report) = ExecutionRequest::serial(&host)
            .run(ExecutionWorkCeiling::new(None), |meter| {
                // Deliberate old behaviour: charge an actual visit to every
                // unrelated item instead of only the changed item.
                for item in 0..self.items {
                    meter.checkpoint(1).unwrap();
                    std::hint::black_box(item);
                }
            })
            .unwrap();
        vec![PhaseWork {
            phase: self.phase,
            charged_work: report.charged_work(),
        }]
    }
}

#[test]
fn an_injected_whole_state_loop_is_caught_for_each_phase_and_axis() {
    let base = Dimensions {
        records: 128,
        history: 128,
        installed: 128,
    };
    for phase in [
        Phase::WholeCommit,
        Phase::CommitPreparation,
        Phase::Publication,
        Phase::RootAdmission,
        Phase::IndexMaintenance,
        Phase::Invariants,
        Phase::Rebase,
        Phase::ReadAdmission,
        Phase::SnapshotOpen,
        Phase::HistoryRecord,
        Phase::Navigation,
    ] {
        for axis in [Axis::Model, Axis::History, Axis::Installed] {
            let samples = base.points().map(|dimensions| {
                let items = match axis {
                    Axis::Model => dimensions.records,
                    Axis::History => dimensions.history,
                    Axis::Installed => dimensions.installed,
                };
                let mut sample = Sample::new(dimensions);
                sample.observe_phase_work(&InjectedFixture { phase, items });
                sample
            });
            let failures = compare(&samples);
            assert!(
                failures
                    .iter()
                    .any(|failure| failure.axis == axis && failure.metric.phase == phase),
                "whole-state mutant escaped {phase:?}/{axis:?}"
            );
            let lawful = base.points().map(|dimensions| {
                let mut sample = Sample::new(dimensions);
                sample.observe_phase_work(&InjectedFixture { phase, items: 1 });
                sample
            });
            assert!(
                compare(&lawful).is_empty(),
                "fixed-change counterpart must pass"
            );
        }
    }
}
