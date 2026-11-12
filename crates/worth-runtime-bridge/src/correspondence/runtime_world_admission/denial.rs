/// A Runtime World correspondence admission failed before any component
/// effect. No fallback correspondence or equal-looking descriptor is valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeWorldCorrespondenceAdmissionDenial {
    ForeignBridgeRuntime {
        expected_runtime_key: u64,
        actual_runtime_key: u64,
    },
    InstalledCorrespondenceNotCurrent,
    AuthoredOwnerUnavailable,
    SignalBranchBasisRequired,
    AuthoredBranchUnavailable {
        signal_graph_instance_id: u64,
    },
    InstalledGenerationDrift {
        expected_generation: u64,
        actual_generation: u64,
    },
}

impl std::fmt::Display for RuntimeWorldCorrespondenceAdmissionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ForeignBridgeRuntime {
                expected_runtime_key,
                actual_runtime_key,
            } => write!(
                formatter,
                "installed correspondence belongs to Bridge runtime {actual_runtime_key}, expected {expected_runtime_key}"
            ),
            Self::SignalBranchBasisRequired => formatter.write_str("authored correspondence requires the actual admitted Signal branch basis"),
            Self::AuthoredOwnerUnavailable => formatter.write_str("authored Bridge owner is unavailable"),
            Self::AuthoredBranchUnavailable { signal_graph_instance_id } => write!(formatter,
                "authored Bridge branch for Signal graph {signal_graph_instance_id} is not serving"),
            Self::InstalledCorrespondenceNotCurrent => {
                formatter.write_str("installed correspondence is not current in its Bridge runtime")
            }
            Self::InstalledGenerationDrift {
                expected_generation,
                actual_generation,
            } => write!(
                formatter,
                "installed correspondence generation {actual_generation} is stale; current generation is {expected_generation}"
            ),
        }
    }
}

impl std::error::Error for RuntimeWorldCorrespondenceAdmissionDenial {}
