use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Phase {
    WholeCommit,
    CommitPreparation,
    Publication,
    RootAdmission,
    IndexMaintenance,
    Invariants,
    Rebase,
    ReadAdmission,
    SnapshotOpen,
    HistoryRecord,
    Navigation,
}

/// The court's consumption interface for the pending owner-issued phase seam.
/// This is evidence only: it cannot authorize, charge or change runtime work.
#[derive(Clone, Copy, Debug)]
pub struct PhaseWork {
    pub phase: Phase,
    pub charged_work: u64,
}

pub trait PhaseWorkSource {
    fn phase_work(&self) -> Vec<PhaseWork>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Metric {
    pub phase: Phase,
    pub name: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Dimensions {
    pub records: usize,
    pub history: usize,
    pub installed: usize,
}

impl Dimensions {
    pub fn points(self) -> [Self; 4] {
        [
            self,
            Self {
                records: self.records * 2,
                ..self
            },
            Self {
                history: self.history * 2,
                ..self
            },
            Self {
                installed: self.installed * 2,
                ..self
            },
        ]
    }
}

#[derive(Debug)]
pub struct Sample {
    pub dimensions: Dimensions,
    pub work: BTreeMap<Metric, u64>,
    pub commits: u64,
    pub touched: Vec<usize>,
}

impl Sample {
    pub fn new(dimensions: Dimensions) -> Self {
        Self {
            dimensions,
            work: BTreeMap::new(),
            commits: 0,
            touched: Vec::new(),
        }
    }

    pub fn record(&mut self, phase: Phase, name: &'static str, units: u64) {
        assert!(
            self.work.insert(Metric { phase, name }, units).is_none(),
            "duplicate observation: {phase:?}/{name}"
        );
    }

    pub fn observe_phase_work(&mut self, source: &impl PhaseWorkSource) {
        for observation in source.phase_work() {
            self.record(observation.phase, "charged_work", observation.charged_work);
        }
    }
}
