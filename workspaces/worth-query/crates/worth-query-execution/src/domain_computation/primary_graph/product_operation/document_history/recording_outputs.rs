//! Actual receipt-derived origin and complete output custody for one recording.
use super::*;
use crate::domain_computation::primary_graph::{
    application_output_demand::OriginalOutputTree, structural_walk::memory,
    WorthQueryAdvancementPhase, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationReadObservation,
    WorthQueryCompletedRequiredOutputs, WorthQueryExpectedProductBranch,
    WorthQueryOriginalReadOutputDenial, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind, WorthQueryOutputDemandSettlement,
    WorthQueryOutputSettlementPosture,
};
use crate::publication_boundary::{
    discovery_currentness::WorthQueryRetainedQueryDependencies, WorthQueryProgramPublicationAccess,
};
use std::sync::Arc;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition,
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
mod completion;
mod source;
mod verification;

enum RecordingTreeCompletion {
    Original(OriginalOutputTree),
    VerifiedLeaf,
    SourceOutsideDomain,
    SourceNotLive,
}

/// Readonly provenance of the original action completion before extra tree work.
/// It grants no query, producer, mutation or adoption permission.
pub struct WorthQueryHistoryRecordingOutputOrigin {
    node: WorthQueryHistoryNodeId,
    candidate: crate::basis::WorthQueryProductBranch,
    expected: crate::basis::WorthQueryProductBranchReadIdentity,
    intent: WorthQueryApplicationIdempotencyBinding,
    source: worth_runtime_world::facade::CompositeCommitIdentity,
    observation: Arc<WorthQueryApplicationReadObservation>,
    profile: WorthQueryHistoryOutputProfile,
    root_posture: Option<WorthQueryOutputSettlementPosture>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

/// One verified recording entry, distinct from a forest's final qualification.
pub struct WorthQueryRetainedRecordingOutputTree {
    origin: Arc<WorthQueryHistoryRecordingOutputOrigin>,
    index: usize,
    completion: RecordingTreeCompletion,
    _verification_memory: Option<worth_execution::ExecutionMemoryReservation>,
}

/// Recording completeness preserves its own Native custody and control causes.
#[derive(Debug)]
pub enum WorthQueryRecordingOutputQualificationDenial {
    Original(WorthQueryOriginalReadOutputDenial),
    Ownership(Box<worth_relational::facade::branch::RelationalModelOwnershipDenial>),
    TreeChanged {
        source: worth_relational::facade::identity::EntityId,
        family: &'static str,
    },
    Incomplete {
        model: worth_relational::facade::identity::EntityId,
    },
}
impl From<WorthQueryOriginalReadOutputDenial> for WorthQueryRecordingOutputQualificationDenial {
    fn from(cause: WorthQueryOriginalReadOutputDenial) -> Self {
        Self::Original(cause)
    }
}

/// Every configured tree qualified at the one receipt-derived final endpoint.
pub struct WorthQueryCompletedRecordingOutputs {
    origin: Arc<WorthQueryHistoryRecordingOutputOrigin>,
    endpoint: Arc<WorthQueryApplicationReadObservation>,
    _trees: Vec<Arc<WorthQueryRetainedRecordingOutputTree>>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl WorthQueryHistoryRecordingOutputOrigin {
    pub fn matches_profile(&self, profile: &WorthQueryHistoryOutputProfile) -> bool {
        self.profile.same_custody(profile)
    }
    pub fn observation(&self) -> &Arc<WorthQueryApplicationReadObservation> {
        &self.observation
    }
    pub fn candidate(&self) -> crate::basis::WorthQueryProductBranch {
        self.candidate
    }
}

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    /// Capture the actual ordinary completion while it is still the candidate's
    /// endpoint. A later arbitrary head cannot stand in for this origin.
    pub fn prepare_history_output_recording(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &WorthQueryProgramPublicationAccess,
        reserved: &WorthQueryDocumentHistoryReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
        source: &WorthQueryApplicationCommitReceipt,
        required: Option<&WorthQueryCompletedRequiredOutputs>,
        profile: &WorthQueryHistoryOutputProfile,
    ) -> Result<Arc<WorthQueryHistoryRecordingOutputOrigin>, WorthQueryDocumentHistoryDenial> {
        use WorthQueryDocumentHistoryDenial as Denial;
        if !profile.is_output_forest() || !profile.matches_recording_model(&self.model) {
            return Err(Denial::ForeignModel);
        }
        let completion = match required {
            Some(completed) => sealing::RecordingCompletion::RequiredOutputs { source, completed },
            None => sealing::RecordingCompletion::Source(source),
        };
        let prepared = self.prepare_binding(phase, reserved, candidate, completion)?;
        let current = self
            .application()
            .on_branch(candidate.branch())
            .select(phase)
            .map_err(Denial::CandidateSelection)?;
        if current.product().observation() != prepared.source.product_observation() {
            return Err(Denial::ReceiptMismatch);
        }
        let execution = phase
            .checked_execution_request_for(&self.application().runtime.product_runtime)
            .map_err(Denial::Stopped)?;
        let memory = memory::reserve(
            &execution,
            execution.has_accounting(),
            (std::mem::size_of::<WorthQueryHistoryRecordingOutputOrigin>()
                + 2 * std::mem::size_of::<usize>()) as u64,
        )
        .map_err(Denial::Recording)?;
        Ok(Arc::new(WorthQueryHistoryRecordingOutputOrigin {
            node: reserved.record.node,
            candidate: candidate.branch(),
            expected: reserved.expected.clone(),
            intent: source.idempotency_binding(),
            source: source
                .committed_product_publication()
                .composite_commit()
                .clone(),
            observation: prepared.source,
            profile: profile.clone(),
            root_posture: prepared.node.root_posture,
            _memory: memory,
        }))
    }
}

impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    crate::domain_computation::primary_graph::application_installation::WorthQueryProgramApplicationRuntime<S, P>
{
    /// Retain the actual closed typed tree, leaving discovery custody with its
    /// original owner on refusal. Its start was checked against this origin.
    pub fn retain_original_recording_output_tree<R: ApplicationOutputGraphShape<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        origin: &Arc<WorthQueryHistoryRecordingOutputOrigin>,
        index: usize,
        root: Arc<WorthQueryOutputDemandSettlement>,
        dependents: impl ExactSizeIterator<Item = Arc<WorthQueryOutputDemandSettlement>>,
        dependencies: &mut Option<WorthQueryRetainedQueryDependencies>,
    ) -> Result<Arc<WorthQueryRetainedRecordingOutputTree>, WorthQueryOriginalReadOutputDenial> {
        let profile = origin.profile.tree_profiles().get(index).ok_or_else(foreign)?;
        if !profile.matches_recording_root::<S, P, R>(self) || root.output_family_identity() != profile.recording_family() {
            return Err(foreign());
        }
        self.validate_selected_output_root::<R>(phase, access, &origin.observation)
            .map_err(WorthQueryOriginalReadOutputDenial::Program)?;
        let execution = phase.checked_execution_request_for(&self.runtime().runtime.product_runtime)
            .map_err(stopped)?;
        let original = OriginalOutputTree::retain(&execution, &origin.observation, root,
            dependents, dependencies, std::mem::size_of::<WorthQueryRetainedRecordingOutputTree>() + 2 * std::mem::size_of::<usize>())?;
        Ok(Arc::new(WorthQueryRetainedRecordingOutputTree { origin: Arc::clone(origin), index, completion: RecordingTreeCompletion::Original(original), _verification_memory:None }))
    }
}
fn foreign() -> WorthQueryOriginalReadOutputDenial {
    WorthQueryOriginalReadOutputDenial::Output(WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::ForeignSource,
        "history output recording origin",
    ))
}
fn stopped(
    cause: crate::domain_computation::primary_graph::WorthQueryAdvancementDenial,
) -> WorthQueryOriginalReadOutputDenial {
    WorthQueryOriginalReadOutputDenial::Output(WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::ExecutionRequest(cause),
        "history output recording",
    ))
}
fn resource(cause: Cause) -> WorthQueryOriginalReadOutputDenial {
    WorthQueryOriginalReadOutputDenial::Output(
        crate::domain_computation::primary_graph::output_reuse::read_denial(cause),
    )
}
fn checkpoint(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), WorthQueryOriginalReadOutputDenial> {
    checkpoint_work(execution, 1)
}
fn checkpoint_work(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    work: u64,
) -> Result<(), WorthQueryOriginalReadOutputDenial> {
    execution
        .run_native_with_checkpoint(|context| {
            crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, work)
        })
        .map_err(|cause| resource(cause.into()))?
        .0
        .map_err(|stop| resource(worth_execution::WorkCeilingDenial::Stopped(stop).into()))
}
