//! A workflow condition authored as an expression over two typed query
//! operands, accepted through the ordinary installed application entry.

use worth_foundational::expression_api::ExpressionDenialFamily;
use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, RequiredWorkflowCondition, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPublicationOutcome, WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorkflowProposalOutcome, WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryWorkflowAdvancePreparationDenial, WorthQueryWorkflowConditionAcceptanceDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationUncommitted,
};

use super::document_retention_model::{
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
const RETENTION_RULE: &str = "uint64(30) / days >= uint64(5) && retained";

macro_rules! read {
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

macro_rules! days {
    ($application:expr) => {
        read!(
            $application,
            DocumentRetentionDaysRead {
                identity: DOCUMENT_IDENTITY.to_owned(),
            }
        )
    };
}

macro_rules! retained {
    ($application:expr) => {
        read!(
            $application,
            DocumentRetentionConditionRead {
                identity: DOCUMENT_IDENTITY.to_owned(),
            }
        )
    };
}

/// Prepares the advance under the operator and evaluates `$body` with it
/// bound to `$prepared`, while the request's principal and scope live.
macro_rules! prepare {
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

macro_rules! accept {
    ($court:expr, $key:expr, days: $days:expr, retained: $retained:expr) => {
        prepare!($court, $key, |prepared| prepared
            .expect("the condition acceptance prepares")
            .condition(&$court.required)
            .operand::<DocumentRetentionDaysQueryBinding, _>("days", $days)
            .operand::<DocumentRetentionConditionQueryBinding, _>("retained", $retained)
            .accept())
    };
}

struct Court {
    application: DocumentWorkflowRuntime,
    instance: PublishedWorkflowInstanceRef,
    required: RequiredWorkflowCondition,
}

impl Court {
    /// A published rule instance awaiting its condition over a document
    /// retained for `days`.
    fn open(days: u64, key: u64) -> Self {
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

    fn assert_terminal(&self, key: u64, path: &str) {
        match advance_instance(&self.application, self.instance.clone(), key)
            .expect("the terminal prepares")
        {
            WorkflowProgressOutcome::Completed(performed) => {
                assert_eq!(performed.node_path(), path)
            }
            other => panic!("expected the {path} terminal, got {other:?}"),
        }
    }

    fn assert_still_awaiting(&self, key: u64) {
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

fn set_days(application: &DocumentWorkflowRuntime, days: u64, key: u64) {
    let program = application.program_runtime();
    assert_eq!(
        settle(set_retention(program, program.current_world(), days, key)),
        RetentionVerdict::Performed(days)
    );
}

fn change_advance_grant(court: &Court, status: &str, key: u64) {
    let runtime = court.application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let outcome = runtime
        .request(&principal, &scope)
        .on_branch(court.instance.branch())
        .mutate(WorkflowGrantStatusIntent {
            input: WorkflowGrantStatusInput {
                grant_identity: "workflow-advance-grant".to_owned(),
                status: status.to_owned(),
            },
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(court.application.program_runtime())
        .expect("the grant change prepares");
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "the grant change must publish: {outcome:?}"
    );
}

fn attempt_kind(
    denial: Result<WorkflowProgressOutcome, WorthQueryWorkflowConditionAcceptanceDenial>,
) -> WorthQueryApplicationAttemptDenialKind {
    match denial {
        Err(WorthQueryWorkflowConditionAcceptanceDenial::Attempt(denial)) => denial.kind(),
        other => panic!("expected an attempt denial, got {other:?}"),
    }
}

#[test]
fn true_expression_selects_satisfied_and_its_retry_replays() {
    let court = Court::open(5, 9_176_100);
    let first = accept!(court, 9_176_110, days: days!(court.application), retained: retained!(court.application))
        .expect("a true expression settles");
    match first {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "positive-retention");
            assert!(!performed.replayed());
        }
        other => panic!("expected a condition settlement, got {other:?}"),
    }
    let retry = accept!(court, 9_176_110, days: days!(court.application), retained: retained!(court.application))
        .expect("the retried acceptance resolves");
    match retry {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "positive-retention");
            assert!(performed.replayed());
        }
        other => panic!("expected a replayed settlement, got {other:?}"),
    }
    court.assert_terminal(9_176_111, "satisfied");
}

#[test]
fn false_expression_selects_unsatisfied() {
    let court = Court::open(8, 9_176_200);
    let days = days!(court.application);
    let retained = retained!(court.application);
    assert_eq!(days.rows(), &[8]);
    assert_eq!(retained.rows(), &[true]);
    assert!(matches!(
        accept!(court, 9_176_210, days: days, retained: retained),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    court.assert_terminal(9_176_211, "unsatisfied");
}

#[test]
fn division_by_zero_denies_without_selecting_false() {
    let court = Court::open(0, 9_176_300);
    let denied = accept!(court, 9_176_310, days: days!(court.application), retained: retained!(court.application));
    let Err(WorthQueryWorkflowConditionAcceptanceDenial::Attempt(denial)) = denied else {
        panic!("a denied expression must refuse the acceptance: {denied:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::WorkflowConditionExpressionDenied
    );
    assert_eq!(
        denial.expression().map(|expression| expression.family()),
        Some(ExpressionDenialFamily::DivisionByZero)
    );
    court.assert_still_awaiting(9_176_311);
}

#[test]
fn absent_or_misnamed_operands_are_not_the_requirement() {
    let court = Court::open(5, 9_176_400);
    let only_retained = prepare!(court, 9_176_410, |prepared| prepared
        .expect("the condition acceptance prepares")
        .condition(&court.required)
        .operand::<DocumentRetentionConditionQueryBinding, _>(
            "retained",
            retained!(court.application),
        )
        .accept());
    assert!(matches!(
        only_retained,
        Err(WorthQueryWorkflowConditionAcceptanceDenial::RequirementMismatch)
    ));
    let misnamed = prepare!(court, 9_176_411, |prepared| prepared
        .expect("the condition acceptance prepares")
        .condition(&court.required)
        .operand::<DocumentRetentionDaysQueryBinding, _>("retention", days!(court.application))
        .operand::<DocumentRetentionConditionQueryBinding, _>(
            "retained",
            retained!(court.application),
        )
        .accept());
    assert!(matches!(
        misnamed,
        Err(WorthQueryWorkflowConditionAcceptanceDenial::RequirementMismatch)
    ));
    court.assert_still_awaiting(9_176_412);
}

#[test]
fn stale_operand_denies_and_fresh_operands_decide() {
    let court = Court::open(5, 9_176_500);
    let days = days!(court.application);
    let retained = retained!(court.application);
    set_days(&court.application, 8, 9_176_510);
    assert_eq!(
        attempt_kind(accept!(court, 9_176_511, days: days, retained: retained)),
        WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch
    );
    court.assert_still_awaiting(9_176_512);
    assert!(matches!(
        accept!(court, 9_176_513, days: days!(court.application), retained: retained!(court.application)),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    court.assert_terminal(9_176_514, "unsatisfied");
}

#[test]
fn source_change_after_admission_is_fenced_at_publication() {
    let court = Court::open(5, 9_176_600);
    let days = days!(court.application);
    let retained = retained!(court.application);
    let raced = prepare!(court, 9_176_610, |prepared| {
        let admitted = prepared.expect("the condition acceptance prepares");
        set_days(&court.application, 8, 9_176_611);
        admitted
            .condition(&court.required)
            .operand::<DocumentRetentionDaysQueryBinding, _>("days", days)
            .operand::<DocumentRetentionConditionQueryBinding, _>("retained", retained)
            .accept()
    });
    match raced {
        Ok(WorkflowProgressOutcome::Application(WorthQueryApplicationUncommitted::Denied(
            denial,
        ))) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::ProductBasisStale
        ),
        other => panic!("a transition over a replaced source must not publish: {other:?}"),
    }
    court.assert_still_awaiting(9_176_612);
}

#[test]
fn stale_zero_divisor_denies_as_stale_before_evaluation() {
    let court = Court::open(0, 9_176_900);
    let days = days!(court.application);
    let retained = retained!(court.application);
    set_days(&court.application, 5, 9_176_910);
    assert_eq!(
        attempt_kind(accept!(court, 9_176_911, days: days, retained: retained)),
        WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch
    );
    court.assert_still_awaiting(9_176_912);
}

#[test]
fn revoked_authority_cannot_accept_the_condition() {
    let court = Court::open(5, 9_176_700);
    change_advance_grant(&court, "revoked", 9_176_710);
    assert!(prepare!(court, 9_176_711, |prepared| matches!(
        prepared,
        Err(WorthQueryWorkflowAdvancePreparationDenial::AwaitingActor(_))
    )));
    change_advance_grant(&court, "active", 9_176_712);
    assert!(matches!(
        accept!(court, 9_176_713, days: days!(court.application), retained: retained!(court.application)),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    court.assert_terminal(9_176_714, "satisfied");
}

#[test]
fn foreign_world_operands_are_refused() {
    let court = Court::open(SEED_RETENTION, 9_176_800);
    let foreign = publish_workflow_on_first_program();
    assert_eq!(
        attempt_kind(accept!(court, 9_176_810, days: days!(foreign), retained: retained!(foreign))),
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch
    );
    court.assert_still_awaiting(9_176_811);
}

#[path = "condition_races.rs"]
mod races;
