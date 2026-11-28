mod binding;
mod contracts;
mod counters;
mod declaration;
mod declaration_basis;
mod discard;
mod execution;
mod promotion;
mod promotion_basis;
mod replay;
mod session;
mod taxonomy;
mod validation;

pub use binding::{
    BranchForkPosture, BridgeSignalBranchIdentity, BridgeSpeculativeBranchBinding,
    BridgeSpeculativeBranchBindingIdentity, RelationalForkAxis, SignalForkAxis,
};
pub use contracts::{BridgePreviewReuseEquivalence, BridgePromotionAdmissibilityProof};
pub use counters::BridgeSpeculationCounters;
pub use declaration::{BridgePreviewSessionDeclaration, BridgePreviewSessionDeclarationIdentity};
pub use declaration_basis::{
    BridgePreviewRetainedArtifactSchema, BridgePreviewSessionBasis, BridgePreviewStructuralBasis,
};
pub use discard::{
    BridgePreviewDiscardCleanupOutcome, BridgePreviewDiscardRecord,
    BridgePreviewDiscardRecordIdentity, BridgePreviewResidueReport,
};
pub use execution::BridgePreviewExecutionRecord;
pub use promotion::{BridgePreviewPromotionRecord, BridgePreviewPromotionRecordIdentity};
pub use promotion_basis::BridgePreviewPromotionAuthorityBasis;
pub use replay::BridgePreviewReplayBundle;
pub(crate) use session::PreviewSessionActivation;
pub use session::{
    BridgePreviewSession, BridgePreviewSessionIdentity, PreviewExecutionRecordIdentity,
};
pub use taxonomy::{
    BridgePreviewLifecycleStateKind, BridgePreviewLifecycleTransitionKind,
    BridgePreviewResidueClass, BridgeRequestKind, BridgeSpeculationFailureClass, PreviewActive,
    PreviewAdmitted, PreviewDeclared, PreviewDiscarded, PreviewPromoted,
};
pub use validation::ValidatedBridgePreviewSessionDeclaration;
