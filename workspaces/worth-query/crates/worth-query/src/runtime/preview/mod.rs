use std::collections::{BTreeMap, BTreeSet};

use super::{
    admit_authority_requirements, validate_inputs, WorthQueryAspectMutationBuilder,
    WorthQueryAspectTouch, WorthQueryAuthorityLane, WorthQueryBatchWriteReceipt,
    WorthQueryDeleteMutationBuilder, WorthQueryDerivedViewHandle, WorthQueryDerivedViewRuntime,
    WorthQueryEffectAction, WorthQueryEffectAdmission, WorthQueryEffectHandle,
    WorthQueryEffectInspectionEvidence, WorthQueryEffectPolicy, WorthQueryEffectTriggerSourceKind,
    WorthQueryExistingTruthTargetBinding, WorthQueryInstalledOperation,
    WorthQueryIntentDeclaration, WorthQueryIntentDenialEvidence, WorthQueryIntentSourceLane,
    WorthQueryLiveView, WorthQueryMutationBatchBuilder, WorthQueryMutationMetadata,
    WorthQueryOperationInput, WorthQueryOperationOutput, WorthQueryPatchBatch,
    WorthQueryPreviewBasisAdmission, WorthQueryPreviewIntentReceipt, WorthQueryProgramEffect,
    WorthQueryProgramTrace, WorthQueryRunReceipt, WorthQueryRuntime, WorthQueryRuntimeError,
    WorthQueryRuntimeFacadeFamily, WorthQuerySameBatchSymbolicTarget,
    WorthQuerySameBatchSymbolicTargetKey, WorthQuerySymbolicTargetReference,
    WorthQuerySymbolicTargetReferenceDenial, WorthQuerySymbolicTargetReferenceDenialKind,
    WorthQueryWriteCommand, WorthQueryWriteReceipt,
};

use crate::declarative_live::DeclarativeLiveQueryRequest;
use crate::memory_workspace::{
    WorthQueryMutationDelta, WorthQueryMutationKind, WorthQuerySnapshotIdentity,
};
use crate::session_label::WorthQuerySessionLabel;
mod aspects;
mod basics;
mod binding;
mod evidence;
mod mutation_ops;
mod outcome;
mod session_closeout;
mod session_execution;
mod symbolic_targets;
mod workflow_ops;

use aspects::{relevant_computed_aspects, relevant_effect_aspects, relevant_live_aspects};
pub use binding::{
    WorthQueryPreviewEffectBindingDisposition, WorthQueryPreviewHandleBindingEvidence,
    WorthQueryPreviewHandleBindingFamily,
};
pub use evidence::{
    WorthQueryPreviewCloseoutEvidence, WorthQueryPreviewCloseoutKind,
    WorthQueryPreviewExecutionEvidence, WorthQueryPreviewExecutionKind,
    WorthQueryPreviewPromotionDenialEvidence, WorthQueryPreviewPromotionDenialKind,
    WorthQueryPreviewResidueClass,
};
pub use outcome::{WorthQueryPreviewDiff, WorthQueryPreviewOutcome};

pub struct WorthQueryPreviewSession<'a> {
    label: WorthQuerySessionLabel,
    runtime: &'a mut WorthQueryRuntime,
    effect_policy: WorthQueryEffectPolicy,
    basis_admission: WorthQueryPreviewBasisAdmission,
    basis_snapshot_identity: WorthQuerySnapshotIdentity,
    pending_commands: Vec<WorthQueryWriteCommand>,
    writes: Vec<WorthQueryWriteReceipt>,
    handle_bindings: Vec<WorthQueryPreviewHandleBindingEvidence>,
    execution_evidence: Vec<WorthQueryPreviewExecutionEvidence>,
    intent_receipts: Vec<WorthQueryPreviewIntentReceipt>,
    promoted: bool,
    discarded: bool,
}

impl<'a> WorthQueryPreviewSession<'a> {
    pub(super) fn new(
        label: WorthQuerySessionLabel,
        runtime: &'a mut WorthQueryRuntime,
        effect_policy: WorthQueryEffectPolicy,
        basis_admission: WorthQueryPreviewBasisAdmission,
    ) -> Result<Self, WorthQueryRuntimeError> {
        let basis_snapshot_identity = runtime.current_snapshot_identity()?;
        Ok(Self {
            label,
            runtime,
            effect_policy,
            basis_admission,
            basis_snapshot_identity,
            pending_commands: Vec::new(),
            writes: Vec::new(),
            handle_bindings: Vec::new(),
            execution_evidence: Vec::new(),
            intent_receipts: Vec::new(),
            promoted: false,
            discarded: false,
        })
    }
}
