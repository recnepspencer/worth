//! A workflow condition authored as an expression over two typed query
//! operands, accepted through the ordinary installed application entry.

pub use worth_foundational::expression_api::ExpressionDenialFamily;
pub use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, RequiredWorkflowCondition, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPublicationOutcome, WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorkflowProposalOutcome, WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryWorkflowAdvancePreparationDenial, WorthQueryWorkflowConditionAcceptanceDenial,
};
pub use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationUncommitted,
};

pub use super::document_retention_model::{
    host::{publish_workflow_on_first_program, DocumentWorkflowRuntime, SEED_RETENTION},
    operator_identity::{authenticate_operator, request_scope},
    presented_request::set_retention,
    retention_days::{DocumentRetentionDaysQueryBinding, DocumentRetentionDaysRead},
    retention_entry::{
        DocumentRetentionConditionQueryBinding, DocumentRetentionConditionRead, DOCUMENT_IDENTITY,
    },
    settled_verdict::{settle, RetentionVerdict},
    workflow::{
        advance_instance, expression_condition_terminal_definition, propose_authoring_instance,
        publish_definition, start_instance, WorkflowAdvanceInput, WorkflowAdvanceIntent,
        WorkflowGrantStatusInput, WorkflowGrantStatusIntent,
    },
};

/// Thirty days of review capacity spread over the retention period must
/// leave at least five days per review cycle, and the document must be
/// retained at all. Division comes first, so zero days denies rather than
/// short-circuiting to false.
pub const RETENTION_RULE: &str = "uint64(30) / days >= uint64(5) && retained";

#[macro_export]
macro_rules! worlds_certification_application_graph_expressions_conditions_rs_read {
    ($application:expr, $intent:expr) => {{
        let runtime = $application.runtime();
        let scope = request_scope();
        let principal = authenticate_operator(runtime.installed_schema(), &scope);
        runtime
            .request(&principal, &scope)
            .query($intent)
            .execute()
            .expect("the operand query executes")
    }};
}
pub use crate::worlds_certification_application_graph_expressions_conditions_rs_read as read;

#[macro_export]
macro_rules! worlds_certification_application_graph_expressions_conditions_rs_days {
    ($application:expr) => {
        read!(
            $application,
            DocumentRetentionDaysRead {
                identity: DOCUMENT_IDENTITY.to_owned(),
            }
        )
    };
}
pub use crate::worlds_certification_application_graph_expressions_conditions_rs_days as days;

#[macro_export]
macro_rules! worlds_certification_application_graph_expressions_conditions_rs_retained {
    ($application:expr) => {
        read!(
            $application,
            DocumentRetentionConditionRead {
                identity: DOCUMENT_IDENTITY.to_owned(),
            }
        )
    };
}
pub use crate::worlds_certification_application_graph_expressions_conditions_rs_retained as retained;

/// Prepares the advance under the operator and evaluates `$body` with it
/// bound to `$prepared`, while the request's principal and scope live.
#[macro_export]
macro_rules! worlds_certification_application_graph_expressions_conditions_rs_prepare {
    ($court:expr, $key:expr, |$prepared:ident| $body:expr) => {{
        let runtime = $court.application.runtime();
        let scope = request_scope();
        let principal = authenticate_operator(runtime.installed_schema(), &scope);
        let $prepared = runtime
            .request(&principal, &scope)
            .mutate(WorkflowAdvanceIntent {
                input: WorkflowAdvanceInput {
                    document_identity: DOCUMENT_IDENTITY.to_owned(),
                },
            })
            .without_source()
            .idempotency(&($key as u64))
            .prepare_workflow_advance(&$court.application, $court.instance.clone());
        $body
    }};
}
pub use crate::worlds_certification_application_graph_expressions_conditions_rs_prepare as prepare;

#[macro_export]
macro_rules! worlds_certification_application_graph_expressions_conditions_rs_accept {
    ($court:expr, $key:expr, days: $days:expr, retained: $retained:expr) => {
        prepare!($court, $key, |prepared| prepared
            .expect("the condition acceptance prepares")
            .condition(&$court.required)
            .operand::<DocumentRetentionDaysQueryBinding, _>("days", $days)
            .operand::<DocumentRetentionConditionQueryBinding, _>("retained", $retained)
            .accept())
    };
}
pub use crate::worlds_certification_application_graph_expressions_conditions_rs_accept as accept;

pub struct Court {
    pub application: DocumentWorkflowRuntime,
    pub instance: PublishedWorkflowInstanceRef,
    pub required: RequiredWorkflowCondition,
}

impl Court {
    /// A published rule instance awaiting its condition over a document
    /// retained for `days`.
    pub fn open(days: u64, key: u64) -> Self {
        let application = publish_workflow_on_first_program();
        if days != SEED_RETENTION {
            set_days(&application, days, key);
        }
        let definition = match publish_definition(
            &application,
            expression_condition_terminal_definition(RETENTION_RULE),
            WorkflowDefinitionExpectedPredecessor::Absent,
            key + 1,
        )
        .expect("the expression condition definition prepares")
        {
            WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
            other => panic!("expected a published expression definition, got {other:?}"),
        };
        let instance = match start_instance(&application, definition.definition().clone(), key + 2)
            .expect("the expression instance prepares")
        {
            WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
            other => panic!("expected a started expression instance, got {other:?}"),
        };
        assert!(matches!(
            propose_authoring_instance(&application, instance.clone(), key + 3),
            Ok(WorkflowProposalOutcome::Published(_))
        ));
        let required = match advance_instance(&application, instance.clone(), key + 4)
            .expect("the expression requirement prepares")
        {
            WorkflowProgressOutcome::AwaitingCondition(required) => required,
            other => panic!("expected an expression requirement, got {other:?}"),
        };
        assert_eq!(
            required
                .operands()
                .iter()
                .map(|operand| operand.name())
                .collect::<Vec<_>>(),
            ["days", "retained"]
        );
        Self {
            application,
            instance,
            required,
        }
    }

    pub fn assert_terminal(&self, key: u64, path: &str) {
        match advance_instance(&self.application, self.instance.clone(), key)
            .expect("the terminal prepares")
        {
            WorkflowProgressOutcome::Completed(performed) => {
                assert_eq!(performed.node_path(), path)
            }
            other => panic!("expected the {path} terminal, got {other:?}"),
        }
    }

    pub fn assert_still_awaiting(&self, key: u64) {
        match advance_instance(&self.application, self.instance.clone(), key)
            .expect("the unsettled condition prepares")
        {
            WorkflowProgressOutcome::AwaitingCondition(required) => {
                assert_eq!(required, self.required)
            }
            other => panic!("a refused condition must select nothing, got {other:?}"),
        }
    }
}

pub fn set_days(application: &DocumentWorkflowRuntime, days: u64, key: u64) {
    let program = application.program_runtime();
    assert_eq!(
        settle(set_retention(program, program.current_world(), days, key)),
        RetentionVerdict::Performed(days)
    );
}

pub fn change_advance_grant(court: &Court, status: &str, key: u64) {
    let runtime = court.application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let outcome = runtime
        .request(&principal, &scope)
        .on_branch(court.instance.branch())
        .advanced()
        .mutate(WorkflowGrantStatusIntent {
            input: WorkflowGrantStatusInput {
                grant_identity: "workflow-advance-grant".to_owned(),
                status: status.to_owned(),
            },
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(
            court.application.program_runtime(),
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the grant change prepares");
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "the grant change must publish: {outcome:?}"
    );
}

pub fn attempt_kind(
    denial: Result<WorkflowProgressOutcome, WorthQueryWorkflowConditionAcceptanceDenial>,
) -> WorthQueryApplicationAttemptDenialKind {
    match denial {
        Err(WorthQueryWorkflowConditionAcceptanceDenial::Attempt(denial)) => denial.kind(),
        other => panic!("expected an attempt denial, got {other:?}"),
    }
}

#[path = "condition_races.rs"]
pub mod races;
