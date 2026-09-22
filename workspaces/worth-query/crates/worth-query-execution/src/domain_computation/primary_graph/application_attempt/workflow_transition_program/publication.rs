use worth_query_declaration::facade::application_program::ApplicationProgramRevision;

use super::super::{WorthQueryApplicationCommitOutcome, WorthQueryApplicationEffectProgram};

#[path = "publication/approval.rs"]
mod approval;
#[path = "publication/assessment_evidence.rs"]
mod assessment_evidence;
#[path = "publication/commit.rs"]
mod commit;
#[path = "publication/evidence_readiness.rs"]
mod evidence_readiness;
#[path = "publication/operation.rs"]
mod operation;
#[path = "publication/required_assessment.rs"]
mod required_assessment;
#[path = "publication/transition_replay.rs"]
mod transition_replay;
pub use approval::{RequiredWorkflowApproval, WorkflowApprovalDecision};
pub use assessment_evidence::PerformedWorkflowAssessmentEvidence;
pub(super) use commit::project;
pub use evidence_readiness::RequiredWorkflowEvidence;
pub use operation::{PreparedWorkflowOperation, RequiredWorkflowOperation};
pub use required_assessment::RequiredWorkflowAssessment;
pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) use transition_replay::PreparedWorkflowTransitionReplays;

#[must_use = "prepared workflow advance owns a live application attempt"]
pub enum PreparedWorkflowAdvance<Schema, Operation, Input, Scope> {
    Transition {
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        program_revision: ApplicationProgramRevision,
        transition_identity: String,
        transition_identity_bytes: [u8; 32],
        transition_identity_locator: worth_foundational::facade::AspectFieldLocator,
        assessment_identity_locator: worth_foundational::facade::AspectFieldLocator,
        instance: worth_relational::facade::identity::EntityId,
        node_path: String,
        terminal: bool,
        assessment: Option<PreparedWorkflowAssessmentProjection>,
        supporting_identity: Option<[u8; 32]>,
        operation_receipt_identity: Option<[u8; 32]>,
        progress_update: Option<
            crate::domain_computation::primary_graph::workflow::instance::PreparedWorkflowProgressUpdate,
        >,
        approval: Option<PreparedWorkflowApprovalProjection>,
        approval_identity: Option<[u8; 32]>,
        replays: PreparedWorkflowTransitionReplays,
    },
    AwaitingAssessment(PreparedWorkflowAssessment<Schema, Operation, Input, Scope>),
    AwaitingCondition(PreparedWorkflowCondition<Schema, Operation, Input, Scope>),
    AwaitingOperation(PreparedWorkflowOperation<Schema, Operation, Input, Scope>),
    AwaitingEvidence {
        read_set: super::super::WorthQueryCompleteApplicationReadSet<
            Schema,
            Operation,
            Input,
            Scope,
            super::super::WorthQueryProjectedApplicationMutation,
        >,
        transition_identity_locator: worth_foundational::facade::AspectFieldLocator,
        assessment_identity_locator: worth_foundational::facade::AspectFieldLocator,
        instance: worth_relational::facade::identity::EntityId,
        required: RequiredWorkflowEvidence,
        replays: PreparedWorkflowTransitionReplays,
    },
    AwaitingApproval {
        read_set: super::super::WorthQueryCompleteApplicationReadSet<
            Schema,
            Operation,
            Input,
            Scope,
            super::super::WorthQueryProjectedApplicationMutation,
        >,
        transition_identity_locator: worth_foundational::facade::AspectFieldLocator,
        assessment_identity_locator: worth_foundational::facade::AspectFieldLocator,
        instance: worth_relational::facade::identity::EntityId,
        required: RequiredWorkflowApproval,
        replays: PreparedWorkflowTransitionReplays,
    },
    ReplayOnly {
        read_set: super::super::WorthQueryCompleteApplicationReadSet<
            Schema,
            Operation,
            Input,
            Scope,
            super::super::WorthQueryProjectedApplicationMutation,
        >,
        transition_identity_locator: worth_foundational::facade::AspectFieldLocator,
        assessment_identity_locator: worth_foundational::facade::AspectFieldLocator,
        instance: worth_relational::facade::identity::EntityId,
        approval: Option<PreparedWorkflowApprovalProjection>,
        approval_identity: Option<[u8; 32]>,
        replays: PreparedWorkflowTransitionReplays,
        denial: super::super::WorthQueryApplicationAttemptDenial,
    },
}

#[derive(Clone)]
#[doc(hidden)]
pub struct PreparedWorkflowApprovalProjection {
    pub(super) identity: String,
    pub(super) identity_locator: worth_foundational::facade::AspectFieldLocator,
    pub(super) fields: Vec<(
        worth_foundational::facade::AspectFieldLocator,
        worth_foundational::facade::AspectValue,
    )>,
    pub(super) required_fields: Vec<worth_foundational::facade::AspectFieldLocator>,
    pub(super) decision: WorkflowApprovalDecision,
    pub(super) proposal: worth_relational::facade::identity::EntityId,
    pub(super) evidence: Box<[worth_relational::facade::identity::EntityId]>,
    pub(super) approver: worth_relational::facade::identity::EntityId,
    pub(super) purpose: String,
    pub(super) expiry: u64,
    pub(super) transition_relation: worth_relational::facade::identity::KindId,
    pub(super) proposal_relation: worth_relational::facade::identity::KindId,
    pub(super) evidence_relation: worth_relational::facade::identity::KindId,
}

#[doc(hidden)]
pub struct PreparedWorkflowAssessmentProjection {
    pub(super) identity_locator: worth_foundational::facade::AspectFieldLocator,
    pub(super) identity: String,
    pub(super) intent_identity: [u8; 32],
    pub(super) producer: String,
    pub(super) family: String,
    pub(super) query: String,
    pub(super) parameter_type: String,
    pub(super) result_type: String,
    pub(super) binding: String,
    pub(super) subject: worth_relational::facade::identity::EntityId,
    pub(super) proposal_identity: String,
    pub(super) coverage_identity: String,
    pub(super) source_identity: String,
    pub(super) passing: bool,
    pub(super) publication_identity: String,
    pub(super) output_content_identity: String,
}

pub struct PreparedWorkflowAssessment<Schema, Operation, Input, Scope> {
    pub(super) admitted:
        crate::domain_computation::primary_graph::workflow::instance::AdmittedWorkflowTransition<
            Schema,
            Operation,
            Input,
            Scope,
        >,
    pub(super) required: RequiredWorkflowAssessment,
    pub(super) layout:
        crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
    pub(super) program_revision: ApplicationProgramRevision,
    pub(super) replays: PreparedWorkflowTransitionReplays,
}

pub struct PreparedWorkflowCondition<Schema, Operation, Input, Scope> {
    pub(super) admitted:
        crate::domain_computation::primary_graph::workflow::instance::AdmittedWorkflowTransition<
            Schema,
            Operation,
            Input,
            Scope,
        >,
    pub(super) required: RequiredWorkflowCondition,
    pub(super) layout:
        crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
    pub(super) program_revision: ApplicationProgramRevision,
    pub(super) replays: PreparedWorkflowTransitionReplays,
}

impl<Schema, Operation, Input, Scope> PreparedWorkflowCondition<Schema, Operation, Input, Scope> {
    pub const fn required(&self) -> &RequiredWorkflowCondition {
        &self.required
    }

    pub fn into_required(self) -> RequiredWorkflowCondition {
        self.required
    }
}

impl<Schema, Operation, Input, Scope> PreparedWorkflowAssessment<Schema, Operation, Input, Scope> {
    pub const fn required(&self) -> &RequiredWorkflowAssessment {
        &self.required
    }

    pub fn into_required(self) -> RequiredWorkflowAssessment {
        self.required
    }
}

#[derive(Clone, Debug)]
pub struct RequiredWorkflowCondition {
    pub(super) instance: worth_relational::facade::identity::EntityId,
    pub(super) node_path: String,
    pub(super) transition_identity: String,
    pub(super) occurrence: u64,
    pub(super) query: String,
    pub(super) parameter_type: String,
    pub(super) result_type: String,
    pub(super) binding: String,
}

impl RequiredWorkflowCondition {
    pub(in crate::domain_computation::primary_graph) fn from_selected(
        instance: worth_relational::facade::identity::EntityId,
        node_path: String,
        transition_identity: String,
        occurrence: u64,
        selected: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowCondition,
    ) -> Self {
        Self {
            instance,
            node_path,
            transition_identity,
            occurrence,
            query: selected.query,
            parameter_type: selected.parameter_type,
            result_type: selected.result_type,
            binding: selected.binding,
        }
    }

    pub const fn instance(&self) -> worth_relational::facade::identity::EntityId {
        self.instance
    }
    pub fn node_path(&self) -> &str {
        &self.node_path
    }
    pub fn transition_identity(&self) -> &str {
        &self.transition_identity
    }
    pub const fn occurrence(&self) -> u64 {
        self.occurrence
    }
    pub fn query(&self) -> &str {
        &self.query
    }
    pub fn parameter_type(&self) -> &str {
        &self.parameter_type
    }
    pub fn result_type(&self) -> &str {
        &self.result_type
    }
    pub fn binding(&self) -> &str {
        &self.binding
    }
}

#[derive(Debug)]
pub struct PerformedWorkflowTransition {
    transition: worth_relational::facade::identity::EntityId,
    node_path: String,
    terminal: bool,
    receipt: super::super::WorthQueryApplicationCommitReceipt,
    replayed: bool,
    assessment_evidence: Option<PerformedWorkflowAssessmentEvidence>,
    approval: Option<PerformedWorkflowApproval>,
    operation_receipt_identity: Option<[u8; 32]>,
}

#[derive(Debug)]
pub struct PerformedWorkflowApproval {
    entity: worth_relational::facade::identity::EntityId,
    identity: String,
    decision: WorkflowApprovalDecision,
    proposal: worth_relational::facade::identity::EntityId,
    evidence: Box<[worth_relational::facade::identity::EntityId]>,
    approver: worth_relational::facade::identity::EntityId,
    purpose: String,
    expiry: u64,
}

impl PerformedWorkflowApproval {
    pub const fn entity(&self) -> worth_relational::facade::identity::EntityId {
        self.entity
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub const fn decision(&self) -> WorkflowApprovalDecision {
        self.decision
    }
    pub const fn proposal(&self) -> worth_relational::facade::identity::EntityId {
        self.proposal
    }
    pub fn evidence(&self) -> &[worth_relational::facade::identity::EntityId] {
        &self.evidence
    }
    pub const fn approver(&self) -> worth_relational::facade::identity::EntityId {
        self.approver
    }
    pub fn purpose(&self) -> &str {
        &self.purpose
    }
    pub const fn expiry(&self) -> u64 {
        self.expiry
    }
}

impl PerformedWorkflowTransition {
    pub const fn transition(&self) -> worth_relational::facade::identity::EntityId {
        self.transition
    }

    pub fn node_path(&self) -> &str {
        &self.node_path
    }

    pub const fn terminal(&self) -> bool {
        self.terminal
    }

    pub const fn receipt(&self) -> &super::super::WorthQueryApplicationCommitReceipt {
        &self.receipt
    }

    pub const fn replayed(&self) -> bool {
        self.replayed
    }

    pub const fn assessment_evidence(&self) -> Option<&PerformedWorkflowAssessmentEvidence> {
        self.assessment_evidence.as_ref()
    }

    pub const fn approval(&self) -> Option<&PerformedWorkflowApproval> {
        self.approval.as_ref()
    }

    pub const fn operation_receipt_identity(&self) -> Option<&[u8; 32]> {
        self.operation_receipt_identity.as_ref()
    }
}

#[derive(Debug)]
pub enum WorkflowProgressOutcome {
    AwaitingAssessment(RequiredWorkflowAssessment),
    AwaitingCondition(RequiredWorkflowCondition),
    AwaitingOperation(RequiredWorkflowOperation),
    AwaitingEvidence(RequiredWorkflowEvidence),
    AwaitingApproval(RequiredWorkflowApproval),
    Completed(PerformedWorkflowTransition),
    Application(WorthQueryApplicationCommitOutcome),
    ProjectionDenied(super::super::WorthQueryApplicationCommitReceipt),
    PreparationDenied(super::super::WorthQueryApplicationAttemptDenial),
    IdempotencyDenied(super::super::WorthQueryApplicationIdempotencyResolutionDenial),
}
