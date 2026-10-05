use crate::runtime_policy::SignalRuntimePolicy;
use worth_foundational::ExecutionPosture;

/// Recommended deployment presets for common workload shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalDeploymentPreset {
    /// Request-driven or UI-driven workloads where cheap operational mode matters most.
    WebDevelopment,
    /// Frame-style or editor-style recomputation with lightweight observability.
    GameEngine,
    /// Audit/replay-heavy workloads where richer retained artifacts are worth the cost.
    Fintech,
    /// Heavy investigative or kernel-style workloads with maximal retained detail.
    Kernel,
}

/// Recommended runtime policy and requested execution posture for a deployment preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalDeploymentPlan {
    pub preset: SignalDeploymentPreset,
    pub runtime_policy: SignalRuntimePolicy,
    pub requested_posture: ExecutionPosture,
    pub summary: &'static str,
    pub certification_command: &'static str,
}

impl SignalDeploymentPreset {
    /// Return the recommended runtime policy, posture, summary, and local
    /// certification command for this deployment shape.
    pub fn recommended(self) -> SignalDeploymentPlan {
        match self {
            Self::WebDevelopment => SignalDeploymentPlan {
                preset: self,
                runtime_policy: SignalRuntimePolicy::web_development(),
                requested_posture: ExecutionPosture::Automatic,
                summary: "Low-overhead operational policy with conservative parallelism for request/interaction-driven workloads.",
                certification_command:
                    "bash scripts/ci/run_signal_local_certification.sh web",
            },
            Self::GameEngine => SignalDeploymentPlan {
                preset: self,
                runtime_policy: SignalRuntimePolicy::game_engine(),
                requested_posture: ExecutionPosture::Automatic,
                summary:
                    "Operational-first policy with earlier staged parallelism for frame-style recomputation.",
                certification_command:
                    "bash scripts/ci/run_signal_local_certification.sh game-engine",
            },
            Self::Fintech => SignalDeploymentPlan {
                preset: self,
                runtime_policy: SignalRuntimePolicy::fintech(),
                requested_posture: ExecutionPosture::Automatic,
                summary:
                    "Development-rich policy with stronger replay detail and conservative deterministic parallel admission.",
                certification_command:
                    "bash scripts/ci/run_signal_local_certification.sh fintech",
            },
            Self::Kernel => SignalDeploymentPlan {
                preset: self,
                runtime_policy: SignalRuntimePolicy::kernel(),
                requested_posture: ExecutionPosture::Automatic,
                summary:
                    "Forensic-rich policy with extended observability and conservative full-parallel admission for heavy compute kernels.",
                certification_command:
                    "bash scripts/ci/run_signal_local_certification.sh kernel",
            },
        }
    }
}
