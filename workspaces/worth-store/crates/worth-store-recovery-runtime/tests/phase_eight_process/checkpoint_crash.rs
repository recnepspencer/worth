#[path = "checkpoint_crash/evidence.rs"]
pub(crate) mod evidence;

#[path = "checkpoint_crash/process.rs"]
mod process;

#[path = "checkpoint_crash/case.rs"]
mod case;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckpointCrashStage {
    CandidateCreation,
    CandidateAppend,
    CandidateBindingCompactionHeader,
    CandidateBindingRecord,
    CandidateFooter,
    CandidateSynchronization,
    CandidatePublication,
    NamespaceSynchronization,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CheckpointCrashScenario {
    id: &'static str,
    stage: CheckpointCrashStage,
}

impl CheckpointCrashStage {
    const fn label(self) -> &'static str {
        match self {
            Self::CandidateCreation => "candidate-creation",
            Self::CandidateAppend => "candidate-append",
            Self::CandidateBindingCompactionHeader => "candidate-binding-header",
            Self::CandidateBindingRecord => "candidate-binding-record",
            Self::CandidateFooter => "candidate-footer",
            Self::CandidateSynchronization => "candidate-synchronization",
            Self::CandidatePublication => "candidate-publication",
            Self::NamespaceSynchronization => "namespace-synchronization",
        }
    }
}

const SCENARIO_COUNT: usize = 16;

// The release matrix is an explicit manifest of sixteen replayable scenario
// identities. Each entry names its production seam and receives independent,
// versioned schedule and perturbation seeds derived from that identity.
const SCENARIOS: [CheckpointCrashScenario; SCENARIO_COUNT] = [
    CheckpointCrashScenario {
        id: "c8-checkpoint-candidate-creation-a",
        stage: CheckpointCrashStage::CandidateCreation,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-candidate-append-a",
        stage: CheckpointCrashStage::CandidateAppend,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-binding-header-a",
        stage: CheckpointCrashStage::CandidateBindingCompactionHeader,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-binding-record-a",
        stage: CheckpointCrashStage::CandidateBindingRecord,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-footer-a",
        stage: CheckpointCrashStage::CandidateFooter,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-synchronization-a",
        stage: CheckpointCrashStage::CandidateSynchronization,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-publication-a",
        stage: CheckpointCrashStage::CandidatePublication,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-namespace-a",
        stage: CheckpointCrashStage::NamespaceSynchronization,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-candidate-creation-b",
        stage: CheckpointCrashStage::CandidateCreation,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-candidate-append-b",
        stage: CheckpointCrashStage::CandidateAppend,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-binding-header-b",
        stage: CheckpointCrashStage::CandidateBindingCompactionHeader,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-binding-record-b",
        stage: CheckpointCrashStage::CandidateBindingRecord,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-footer-b",
        stage: CheckpointCrashStage::CandidateFooter,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-synchronization-b",
        stage: CheckpointCrashStage::CandidateSynchronization,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-publication-b",
        stage: CheckpointCrashStage::CandidatePublication,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-namespace-b",
        stage: CheckpointCrashStage::NamespaceSynchronization,
    },
];

#[derive(Debug, Clone, Copy)]
struct ScenarioSeeds {
    schedule: u64,
    perturbation: u64,
}

fn scenario_seeds() -> [ScenarioSeeds; SCENARIO_COUNT] {
    std::array::from_fn(|index| {
        let mut digest = Sha256::new();
        digest.update(b"worth.store.c8.checkpoint-scenario.v2");
        digest.update(SCENARIOS[index].id.as_bytes());
        let digest: [u8; 32] = digest.finalize().into();
        let schedule = u64::from_le_bytes(digest[..8].try_into().unwrap());
        let perturbation = u64::from_le_bytes(digest[8..16].try_into().unwrap());
        ScenarioSeeds {
            schedule,
            perturbation: if perturbation == schedule {
                perturbation.wrapping_add(1)
            } else {
                perturbation
            },
        }
    })
}

#[test]
fn killed_checkpoint_writer_reopens_and_observes_each_persisted_effect_frontier() {
    let seeds = scenario_seeds();
    let mut schedules = std::collections::BTreeSet::new();
    let mut perturbations = std::collections::BTreeSet::new();
    for (index, (scenario, seed)) in SCENARIOS.into_iter().zip(seeds).enumerate() {
        assert!(
            schedules.insert(seed.schedule),
            "duplicate checkpoint schedule seed"
        );
        assert!(
            perturbations.insert(seed.perturbation),
            "duplicate checkpoint perturbation seed"
        );
        case::run_checkpoint_case(index, scenario, seed.schedule, seed.perturbation);
    }
    assert_eq!(schedules.len(), SCENARIO_COUNT);
    assert_eq!(perturbations.len(), SCENARIO_COUNT);
}
use sha2::{Digest, Sha256};
