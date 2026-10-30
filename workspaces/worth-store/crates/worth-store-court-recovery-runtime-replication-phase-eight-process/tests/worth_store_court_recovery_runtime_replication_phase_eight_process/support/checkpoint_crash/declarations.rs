#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CheckpointCrashStage {
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
pub(crate) struct CheckpointCrashScenario {
    pub(crate) id: &'static str,
    pub(crate) stage: CheckpointCrashStage,
}

impl CheckpointCrashStage {
    pub(crate) const fn label(self) -> &'static str {
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

pub(crate) const SCENARIO_COUNT: usize = 16;

// The release matrix is an explicit manifest of sixteen replayable scenario
// identities. Each entry names its production seam and receives independent,
// versioned schedule and perturbation seeds derived from that identity.
pub(crate) const SCENARIOS: [CheckpointCrashScenario; SCENARIO_COUNT] = [
    CheckpointCrashScenario {
        id: "c8-checkpoint-candidate-creation-a",
        stage: crate::checkpoint_crash::CheckpointCrashStage::CandidateCreation,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-candidate-append-a",
        stage: crate::checkpoint_crash::CheckpointCrashStage::CandidateAppend,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-binding-header-a",
        stage: crate::checkpoint_crash::CheckpointCrashStage::CandidateBindingCompactionHeader,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-binding-record-a",
        stage: crate::checkpoint_crash::CheckpointCrashStage::CandidateBindingRecord,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-footer-a",
        stage: crate::checkpoint_crash::CheckpointCrashStage::CandidateFooter,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-synchronization-a",
        stage: crate::checkpoint_crash::CheckpointCrashStage::CandidateSynchronization,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-publication-a",
        stage: crate::checkpoint_crash::CheckpointCrashStage::CandidatePublication,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-namespace-a",
        stage: crate::checkpoint_crash::CheckpointCrashStage::NamespaceSynchronization,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-candidate-creation-b",
        stage: crate::checkpoint_crash::CheckpointCrashStage::CandidateCreation,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-candidate-append-b",
        stage: crate::checkpoint_crash::CheckpointCrashStage::CandidateAppend,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-binding-header-b",
        stage: crate::checkpoint_crash::CheckpointCrashStage::CandidateBindingCompactionHeader,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-binding-record-b",
        stage: crate::checkpoint_crash::CheckpointCrashStage::CandidateBindingRecord,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-footer-b",
        stage: crate::checkpoint_crash::CheckpointCrashStage::CandidateFooter,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-synchronization-b",
        stage: crate::checkpoint_crash::CheckpointCrashStage::CandidateSynchronization,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-publication-b",
        stage: crate::checkpoint_crash::CheckpointCrashStage::CandidatePublication,
    },
    CheckpointCrashScenario {
        id: "c8-checkpoint-namespace-b",
        stage: crate::checkpoint_crash::CheckpointCrashStage::NamespaceSynchronization,
    },
];

#[derive(Debug, Clone, Copy)]
pub(crate) struct ScenarioSeeds {
    pub(crate) schedule: u64,
    pub(crate) perturbation: u64,
}

pub(crate) fn scenario_seeds() -> [ScenarioSeeds; SCENARIO_COUNT] {
    std::array::from_fn(|index| {
        let mut digest = sha2::Sha256::new();
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
pub(crate) use {sha2::Digest, sha2::Sha256};
