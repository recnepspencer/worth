mod constructors;
mod terminal_projection;

use crate::facade::{TruthBranchIdentity, TruthCommitIdentity};
use crate::source::SourceDeclarationIdentity;
use crate::structural::StructuralIdentityDeclarationIdentity;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeHarnessTargetId {
    CommittedRoute {
        commit_identity: TruthCommitIdentity,
    },
    StreamRouting {
        commit_window: Vec<TruthCommitIdentity>,
    },
    StreamReplayAudit {
        commit_window: Vec<TruthCommitIdentity>,
    },
    SourceMaterialize {
        declaration_identity: SourceDeclarationIdentity,
    },
    SourceMaterializeBatch {
        declaration_identity: SourceDeclarationIdentity,
    },
    SourceReplay {
        declaration_identity: SourceDeclarationIdentity,
    },
    SourceRejectUnregistered {
        declaration_identity: SourceDeclarationIdentity,
    },
    SourceRejectOpenSnapshot {
        declaration_identity: SourceDeclarationIdentity,
    },
    SourceRejectSnapshotDrift {
        declaration_identity: SourceDeclarationIdentity,
    },
    PolicyProvenanceCertification,
    PolicyRejectionCertification,
    PolicyAmbientLeakCertification,
    SpeculationDiscardCertification,
    SpeculationPromotionCertification,
    SpeculationChurnCertification,
    StructuralRemapExact {
        declaration_identity: StructuralIdentityDeclarationIdentity,
    },
    StructuralRemapAmbiguous {
        declaration_identity: StructuralIdentityDeclarationIdentity,
    },
    StructuralRemapNoSafeMatch {
        declaration_identity: StructuralIdentityDeclarationIdentity,
    },
    StructuralRemapLineageDivergence {
        declaration_identity: StructuralIdentityDeclarationIdentity,
    },
    StructuralRemapIdentityConflict {
        declaration_identity: StructuralIdentityDeclarationIdentity,
    },
    StructuralRemapReplay {
        declaration_identity: StructuralIdentityDeclarationIdentity,
    },
    StructuralBranchCompare {
        declaration_identity: StructuralIdentityDeclarationIdentity,
    },
    StructuralBranchReplay {
        declaration_identity: StructuralIdentityDeclarationIdentity,
    },
    WritebackDuplicateCertification,
    WritebackAuthorityDenialCertification,
    WritebackFeedbackLoopCertification,
    WritebackReplayMismatchCertification,
    WritebackExtensibleFamilyCertification,
    WritebackMultiFamilyAdmissionBoundaryCertification,
    WritebackCrossFamilyReplayLoopIsolationCertification,
    WritebackHostMapperParityCertification,
    HistoricalCommit {
        branch_identity: TruthBranchIdentity,
        commit_identity: TruthCommitIdentity,
    },
    BranchHead {
        branch_identity: TruthBranchIdentity,
    },
}
