use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

use super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEffectProgram, WorthQueryCompleteApplicationReadSet,
    WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::primary_graph::workflow::{
    definition::{reconstruct_compiled_definition, WorkflowDefinitionCompilationPosture},
    instance::{
        admit_workflow_transition, select_assessment_collection, select_current_transition,
        select_navigation_back_transition, select_settled_replay_transition,
        select_terminal_transition, visit_terminal_transition_facts,
        visit_workflow_transition_facts, SelectedWorkflowTransitionKind, WorkflowInstanceState,
    },
};

pub mod advance;
pub mod assessment_applicability;
pub mod assessment_coverage;
pub mod assessment_materialization;
pub mod await_inbound;
pub mod condition_materialization;
pub mod evidence_join;
pub mod navigation;
pub mod publication;
pub mod terminal;


#[derive(Clone)]
pub enum WorkflowTransitionRequestKind {
    Advance,
    NavigateBack,
    LocalAction {
        kind: WorkflowLocalActionKind,
        requested:
            crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowOperation,
        input_identity: [u8; 32],
        completion: Option<local_actions::VerifiedLocalActionCompletion>,
    },
    CollectAssessment {
        node_path: String,
    },
}

fn denial(
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(kind, subject)
}

pub mod actor_selection;
pub mod approval;
pub mod approval_decision;
pub mod local_actions;
pub use actor_selection::WorkflowRecordedActorObservation;
pub use operation::local_settlement::LocalActionSettlementProof;
pub use publication::{
    PreparedWorkflowAdvance, };
pub mod assessment {
    pub use super::publication::RequiredWorkflowAssessment;
    pub use crate::domain_computation::primary_graph::workflow::assessment::WorkflowAssessmentEvidenceMeaning;
    pub use crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryWorkflowAssessmentPosture;
    pub use crate::domain_computation::primary_graph::{
        WorthQueryObservedSource, WorthQueryOutputDemandSettlement,
    };
    pub mod currentness_facts;
    pub mod evidence_budget;
    pub mod identity;
    pub use currentness_facts::bind_currentness_facts;
}
pub mod operation {
    pub use crate::domain_computation::primary_graph::application_attempt::check_request_live;
    pub use worth_query_declaration::facade::{
        application_operation::{ApplicationMutationBinding, ApplicationMutationIdentities},
        application_schema::ApplicationOperationMarkerIdentity,
    };
    pub use worth_query_installation::facade::ApplicationSchema;
    pub use super::{
        PreparedWorkflowAdvance, PreparedWorkflowOperation, RequiredWorkflowOperation,
        WorkflowOperationAuthority,
    };
    pub use crate::domain_computation::primary_graph::{
        application_attempt::workflow_instance_observation::{
            instance_binding, settlement::observe_retained_transition,
        },
        workflow::instance::WorkflowInstanceProgress,
        workflow::{
            definition::{CompiledWorkflowDefinition, CompiledWorkflowNodeKind},
            proposal::observe_workflow_operation_input,
        },
        WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
        WorthQueryApplicationCommitReceipt, WorthQueryApplicationEffectProgram,
        WorthQueryApplicationObservedFact, WorthQueryCompleteApplicationReadSet,
        WorthQueryProjectedApplicationMutation,
    };
    pub fn mismatch(subject: &str) -> WorthQueryApplicationAttemptDenial {
        WorthQueryApplicationAttemptDenial::new(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
            subject,
        )
    }
    pub mod authority_binding;
    pub mod evidence;
    pub mod local_settlement;
    pub mod materialize;
    pub mod receipt;
    pub use receipt::ValidatedWorkflowOperationReceipt;
}

pub use publication::{
    PreparedWorkflowAssessment, PreparedWorkflowCondition,     RequiredWorkflowApproval, RequiredWorkflowAssessment, RequiredWorkflowCondition,
    RequiredWorkflowEvidence,
};


pub use publication::WorkflowApprovalDecision;

pub use local_actions::WorkflowLocalActionKind;

pub mod actor_context;
pub use actor_context::WorthQueryWorkflowActorContext;
pub use operation::ValidatedWorkflowOperationReceipt;
pub use publication::{PreparedWorkflowOperation, RequiredWorkflowOperation, WorkflowOperationAuthority};

pub use publication::WorkflowProgressOutcome;

