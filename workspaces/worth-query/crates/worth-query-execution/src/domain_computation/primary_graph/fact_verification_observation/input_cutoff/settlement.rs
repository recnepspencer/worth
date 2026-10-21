//! Fixed-size descriptions of the original native no-row answer at a cutoff.
use crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding;
use crate::domain_computation::primary_graph::output_lineage::invalidation::{
    FullVerificationReason, SourceSettlementCurrentness,
};
use worth_relational::facade::{
    history::CommitId, identity::VersionId, runtime::PositionedRelationalSnapshot,
};

/// The native full-verification reason that declined this cutoff's mark answer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInputCutoffNoRowReasonForTest {
    MissingSettlement,
    BeforeReadBasis,
    DifferentBranch,
    CheckpointRestore,
}

/// The native answer, not delivery's separate full-verification result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInputCutoffSettlementPostureForTest {
    Foreign,
    FullVerificationRequired(WorthQueryInputCutoffNoRowReasonForTest),
}

/// Copied from the actual candidate, never reconstructed from affected marks.
/// The receipt's producer supplies its installed producer context. No slot,
/// parameter, binding-text copy or live settlement is retained.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryInputCutoffCandidateForTest {
    pub runtime_authority: u64,
    pub scope: WorthQueryOperationScopeEntityBinding,
    pub product_incarnation: worth_runtime_world::facade::ProductBranchIncarnation,
    pub publication_generation: u64,
}

/// Identity fields of the already-selected native image, without its branch
/// text, snapshot handle, roots or any history obligation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryInputCutoffNativeBasisForTest {
    pub runtime_instance_id: u64,
    pub root_id: u64,
    pub version: VersionId,
    pub commit: Option<CommitId>,
    pub position: Option<worth_relational::facade::publication::PatchStreamPosition>,
}

/// The original native decline and its candidate and selected-image descriptions.
/// These copied observations grant no currentness or selection authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryInputCutoffSettlementForTest {
    pub posture: WorthQueryInputCutoffSettlementPostureForTest,
    pub candidate: WorthQueryInputCutoffCandidateForTest,
    pub selected_native: WorthQueryInputCutoffNativeBasisForTest,
}

pub(in crate::domain_computation::primary_graph) fn settlement_decline(
    currentness: &SourceSettlementCurrentness,
    selected: &PositionedRelationalSnapshot,
    candidate: impl FnOnce() -> WorthQueryInputCutoffCandidateForTest,
) {
    if !super::super::execution_observation::is_installed() {
        return;
    }
    super::CURRENT.with(|current| {
        let Some(mut receipt) = current.get() else {
            return;
        };
        use WorthQueryInputCutoffNoRowReasonForTest as Reason;
        use WorthQueryInputCutoffSettlementPostureForTest as Posture;
        let posture = match currentness {
            SourceSettlementCurrentness::Foreign => Posture::Foreign,
            SourceSettlementCurrentness::FullVerificationRequired(reason) => {
                let reason = match reason {
                    FullVerificationReason::MissingSettlement => Reason::MissingSettlement,
                    FullVerificationReason::BeforeReadBasis => Reason::BeforeReadBasis,
                    FullVerificationReason::DifferentBranch => Reason::DifferentBranch,
                    FullVerificationReason::CheckpointRestore => Reason::CheckpointRestore,
                    FullVerificationReason::NativeRevisionUnavailable
                    | FullVerificationReason::NativeFactRevisionUnavailable(_)
                    | FullVerificationReason::IndexedSelectionDenied(_)
                    | FullVerificationReason::IndexedSelectionFactDenied(_, _)
                    | FullVerificationReason::UnsupportedFact
                    | FullVerificationReason::RetainedDeliveryGap
                    | FullVerificationReason::DeclaredChangeUnavailable
                    | FullVerificationReason::RegistrationIncomplete
                    | FullVerificationReason::ComputationSuperseded => return,
                };
                Posture::FullVerificationRequired(reason)
            }
            SourceSettlementCurrentness::Clean
            | SourceSettlementCurrentness::Dirty(_)
            | SourceSettlementCurrentness::PendingUpstream(_) => return,
        };
        receipt.settlement_decline = Some(WorthQueryInputCutoffSettlementForTest {
            posture,
            candidate: candidate(),
            selected_native: WorthQueryInputCutoffNativeBasisForTest {
                runtime_instance_id: selected.runtime_instance_id(),
                root_id: selected.root_id(),
                version: selected.version_id(),
                commit: selected.commit_id(),
                position: selected.position(),
            },
        });
        current.set(Some(receipt));
    });
}
