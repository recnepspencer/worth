mod constructors;
mod terminal_projection;

use crate::facade::{TruthBranchIdentity, TruthCommitIdentity};
use crate::source::SourceDeclarationIdentity;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeHarnessTargetId {
    CommittedRoute {
        commit_identity: TruthCommitIdentity,
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
    HistoricalCommit {
        branch_identity: TruthBranchIdentity,
        commit_identity: TruthCommitIdentity,
    },
    BranchHead {
        branch_identity: TruthBranchIdentity,
    },
}
