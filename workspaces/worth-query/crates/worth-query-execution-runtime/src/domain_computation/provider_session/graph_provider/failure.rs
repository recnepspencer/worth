#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphCallBindingDenial {
    BoundOperationAuthorityMismatch,
    CommitKindRequiresCommitCall,
    ExecutionBasisMismatch,
    ForeignResourceAttempt,
}

impl WorthQueryGraphCallBindingDenial {
    pub fn detail(&self) -> &'static str {
        match self {
            Self::BoundOperationAuthorityMismatch => {
                "graph call is outside the exact bound operation authority"
            }
            Self::CommitKindRequiresCommitCall => {
                "commit admission requires the graph commit-call authority"
            }
            Self::ExecutionBasisMismatch => {
                "graph call snapshot does not match the running execution basis"
            }
            Self::ForeignResourceAttempt => {
                "execution resources were admitted for another provider session"
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphReceiptAdmissionDenial {
    ForeignCall,
    MissingProjection,
    ProjectionAuthorityMismatch,
    UnexpectedProjection,
}

impl WorthQueryGraphReceiptAdmissionDenial {
    pub fn detail(&self) -> &'static str {
        match self {
            Self::ForeignCall => "graph provider returned a receipt minted for another Query call",
            Self::MissingProjection => {
                "graph projection call returned no execution-bound projection material"
            }
            Self::ProjectionAuthorityMismatch => {
                "graph projection product was sealed by another Query call"
            }
            Self::UnexpectedProjection => {
                "graph provider returned projection material for a non-projection call"
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphProviderFailure {
    detail: String,
}

impl WorthQueryGraphProviderFailure {
    pub fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}
