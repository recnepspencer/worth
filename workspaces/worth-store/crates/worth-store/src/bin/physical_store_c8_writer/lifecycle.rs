use worth_store::physical_runtime::production::PhysicalCheckpointStep;

#[derive(Clone, Copy)]
pub(super) enum CheckpointStage {
    CandidateCreation,
    CandidateAppend,
    CandidateBindingCompactionHeader,
    CandidateBindingRecord,
    CandidateFooter,
    CandidateSynchronization,
    CandidatePublication,
    NamespaceSynchronization,
    NamespaceSynchronizationComplete,
}

impl CheckpointStage {
    pub(super) fn parse(encoded: &str) -> Result<(Self, u64, u64), String> {
        let (label, encoded_seeds) = encoded.split_once('@').ok_or_else(|| {
            "C8 checkpoint stage requires schedule and perturbation seeds".to_owned()
        })?;
        let (schedule_seed, perturbation_seed) = encoded_seeds
            .split_once(':')
            .ok_or_else(|| "C8 checkpoint stage requires distinct seed fields".to_owned())?;
        let schedule_seed = schedule_seed
            .parse::<u64>()
            .map_err(|_| format!("invalid C8 schedule seed `{schedule_seed}`"))?;
        let perturbation_seed = perturbation_seed
            .parse::<u64>()
            .map_err(|_| format!("invalid C8 perturbation seed `{perturbation_seed}`"))?;
        if schedule_seed == perturbation_seed {
            return Err("C8 schedule and perturbation seeds must be distinct".to_owned());
        }
        let stage = match label {
            "candidate-creation" => Self::CandidateCreation,
            "candidate-append" => Self::CandidateAppend,
            "candidate-binding-header" => Self::CandidateBindingCompactionHeader,
            "candidate-binding-record" => Self::CandidateBindingRecord,
            "candidate-footer" => Self::CandidateFooter,
            "candidate-synchronization" => Self::CandidateSynchronization,
            "candidate-publication" => Self::CandidatePublication,
            "namespace-synchronization" => Self::NamespaceSynchronization,
            "namespace-synchronization-complete" => Self::NamespaceSynchronizationComplete,
            _ => return Err(format!("unknown C8 checkpoint stage `{label}`")),
        };
        Ok((stage, schedule_seed, perturbation_seed))
    }

    pub(super) const fn step(self) -> PhysicalCheckpointStep {
        match self {
            Self::CandidateCreation => PhysicalCheckpointStep::CandidateCreation,
            Self::CandidateAppend => PhysicalCheckpointStep::CandidateAppend,
            Self::CandidateBindingCompactionHeader => {
                PhysicalCheckpointStep::CandidateBindingCompactionHeader
            }
            Self::CandidateBindingRecord => PhysicalCheckpointStep::CandidateBindingRecord,
            Self::CandidateFooter => PhysicalCheckpointStep::CandidateFooter,
            Self::CandidateSynchronization => PhysicalCheckpointStep::CandidateSynchronization,
            Self::CandidatePublication => PhysicalCheckpointStep::CandidatePublication,
            Self::NamespaceSynchronization | Self::NamespaceSynchronizationComplete => {
                PhysicalCheckpointStep::NamespaceSynchronization
            }
        }
    }
}

pub(super) fn checkpoint_stage_label(stage: CheckpointStage) -> &'static str {
    match stage {
        CheckpointStage::CandidateCreation => "candidate-creation",
        CheckpointStage::CandidateAppend => "candidate-append",
        CheckpointStage::CandidateBindingCompactionHeader => "candidate-binding-header",
        CheckpointStage::CandidateBindingRecord => "candidate-binding-record",
        CheckpointStage::CandidateFooter => "candidate-footer",
        CheckpointStage::CandidateSynchronization => "candidate-synchronization",
        CheckpointStage::CandidatePublication => "candidate-publication",
        CheckpointStage::NamespaceSynchronization
        | CheckpointStage::NamespaceSynchronizationComplete => "namespace-synchronization",
    }
}
