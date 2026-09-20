use crate::declaration::UiDeclarationGraphHandoffDenial;
use crate::graph::{UiGraphInstantiationDenial, UiGraphMutationCommitDenial};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiApplicationPreparationPhase {
    CandidateBasis,
    GraphHandoff,
    GraphAdmission,
    GraphCommit,
    ServicePolicyNormalization,
}

/// Phase-local denial from the single public application-preparation lane.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthUiApplicationPreparationDenial {
    DslCompilation(Box<worth_ui_dsl::WorthUiDslCompileReport>),
    RuntimePreparation(Box<crate::runtime::WorthUiSemanticHandoffPreparationDenial>),
    Candidate(Box<crate::runtime::WorthUiReplacementCandidateDenial>),
    IntentCatalog(Box<crate::declaration::UiIntentCatalogPreparationDenial>),
    IntentExecutionBinding(
        Box<crate::runtime::intent_execution::UiIntentExecutionBindingPreparationDenial>,
    ),
    CandidateSnapshotMismatch {
        candidate_snapshot_digest: u64,
        prepared_snapshot_digest: u64,
    },
    GraphHandoff(Box<UiDeclarationGraphHandoffDenial>),
    GraphAdmission(Box<UiGraphInstantiationDenial>),
    GraphCommit(Box<UiGraphMutationCommitDenial>),
    ServicePolicyNormalization(crate::declaration::UiServicePolicyNormalizationDenial),
}

impl WorthUiApplicationPreparationDenial {
    pub fn phase(&self) -> WorthUiApplicationPreparationPhase {
        match self {
            Self::DslCompilation(_)
            | Self::RuntimePreparation(_)
            | Self::Candidate(_)
            | Self::IntentCatalog(_)
            | Self::IntentExecutionBinding(_) => WorthUiApplicationPreparationPhase::CandidateBasis,
            Self::CandidateSnapshotMismatch { .. } => {
                WorthUiApplicationPreparationPhase::CandidateBasis
            }
            Self::GraphHandoff(_) => WorthUiApplicationPreparationPhase::GraphHandoff,
            Self::GraphAdmission(_) => WorthUiApplicationPreparationPhase::GraphAdmission,
            Self::GraphCommit(_) => WorthUiApplicationPreparationPhase::GraphCommit,
            Self::ServicePolicyNormalization(_) => {
                WorthUiApplicationPreparationPhase::ServicePolicyNormalization
            }
        }
    }
}
