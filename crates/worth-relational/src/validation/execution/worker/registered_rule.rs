//! Native and custom invariant registration execution.

use crate::validation::data::{
    CustomInvariantExecutionContext, CustomInvariantFailure, CustomInvariantFailureKind,
    CustomInvariantProvenance, CustomInvariantRuntimePhase, InvariantReportedRule,
    InvariantVerdict,
};
use crate::validation::engine::budget::InvariantBudget;
use crate::validation::engine::context::InvariantExecutionContext;
use crate::validation::engine::evaluator::evaluate_rule;
use crate::validation::engine::InvariantRuntimeView;

use super::super::packets::InvariantWorkPacket;

pub(super) struct RegisteredInvariantEvaluation {
    pub(super) reported_rule: InvariantReportedRule,
    pub(super) groups: crate::validation::data::InvariantGroupSet,
    pub(super) cost: crate::validation::data::InvariantCostClass,
    pub(super) custom_provenance: Option<CustomInvariantProvenance>,
    pub(super) verdicts: Vec<InvariantVerdict>,
}

pub(super) fn evaluate_registered_rule<'state>(
    runtime: &InvariantRuntimeView<'state>,
    packet: &InvariantWorkPacket<'state>,
    budget: Option<&dyn InvariantBudget>,
) -> RegisteredInvariantEvaluation {
    match &packet.registration {
        crate::authority::commit::preparation::packets::invariant::InvariantPacketRegistration::Native(
            registration,
        ) => evaluate_native_registration(runtime, packet, registration, budget),
        crate::authority::commit::preparation::packets::invariant::InvariantPacketRegistration::Custom {
            registration,
            prepared_execution,
            prepared_scope,
            retained_touched,
        } => evaluate_custom_registration(
            runtime,
            packet,
            registration,
            prepared_execution,
            prepared_scope,
            retained_touched,
            budget,
        ),
        crate::authority::commit::preparation::packets::invariant::InvariantPacketRegistration::CustomNotApplicable {
            registration,
            prepared_scope,
            work,
        } => {
            let context = CustomInvariantExecutionContext::new(
                runtime,
                packet.observation,
                packet.version_id,
                packet.current_version_id,
                prepared_scope,
                None,
                work.clone(),
                std::sync::Arc::new(registration.access_contract().clone()),
            );
            RegisteredInvariantEvaluation {
                reported_rule: InvariantReportedRule::Custom(registration.descriptor().identity.clone()),
                groups: registration.groups(),
                cost: registration.cost_class(),
                custom_provenance: Some(context.provenance()),
                verdicts: vec![InvariantVerdict::NotApplicable],
            }
        }
    }
}

fn evaluate_native_registration<'state>(
    runtime: &InvariantRuntimeView<'state>,
    packet: &InvariantWorkPacket<'state>,
    registration: &crate::validation::data::InvariantRegistration,
    budget: Option<&dyn InvariantBudget>,
) -> RegisteredInvariantEvaluation {
    let mut context = InvariantExecutionContext::new(
        runtime,
        packet.observation.clone(),
        packet.version_id,
        packet.current_version_id,
        packet.merged_plan,
        packet.relation_integrity_scopes.clone(),
        std::sync::Arc::clone(&packet.current_version_minimum_index),
    );
    if let Some(budget) = budget {
        context = context.with_budget(budget);
    }
    let violations = evaluate_rule(
        &context,
        registration.execution_point.class(),
        &registration.rule,
    );
    let verdicts = if violations.is_empty() {
        vec![InvariantVerdict::Pass]
    } else {
        violations
            .into_iter()
            .map(|violation| registration.verdict_for_violation(violation))
            .collect()
    };
    RegisteredInvariantEvaluation {
        reported_rule: InvariantReportedRule::Native(registration.rule.clone()),
        groups: registration.groups(),
        cost: registration.cost(),
        custom_provenance: None,
        verdicts,
    }
}

fn evaluate_custom_registration<'state>(
    runtime: &InvariantRuntimeView<'state>,
    packet: &InvariantWorkPacket<'state>,
    registration: &crate::validation::data::CustomInvariantRegistration,
    prepared_execution: &std::sync::Arc<
        dyn crate::validation::data::PreparedCustomInvariantExecution,
    >,
    prepared_scope: &crate::validation::data::PreparedCustomInvariantScope,
    retained_touched: &std::sync::Arc<crate::validation::data::TouchedStructuralSet>,
    budget: Option<&dyn InvariantBudget>,
) -> RegisteredInvariantEvaluation {
    let structural_budget = budget.map(InvariantBudget::custom_structural_budget);
    let work = prepared_execution.work_meter();
    let work = structural_budget
        .as_ref()
        .map_or(work.clone(), |structural| work.for_evaluation(structural));
    let ready = budget.is_none_or(|budget| budget.checkpoint(1));
    let context = CustomInvariantExecutionContext::new(
        runtime,
        packet.observation,
        packet.version_id,
        packet.current_version_id,
        prepared_scope,
        Some(std::sync::Arc::clone(retained_touched)),
        work.clone(),
        std::sync::Arc::new(registration.access_contract().clone()),
    );
    let outcome = if !ready
        || structural_budget
            .as_ref()
            .is_some_and(|budget| budget.stop().is_some())
    {
        crate::validation::data::PreparedCustomInvariantExecutionOutcome::Verdict(
            crate::validation::data::CustomInvariantVerdict::Pass,
        )
    } else if let Some(budget) = budget {
        match prepared_execution.evaluate_checked(&context, budget.custom_meter()) {
            Some(outcome) => outcome,
            None => {
                budget.deny_unchecked_custom();
                crate::validation::data::PreparedCustomInvariantExecutionOutcome::Verdict(
                    crate::validation::data::CustomInvariantVerdict::Pass,
                )
            }
        }
    } else {
        prepared_execution.evaluate(&context)
    };
    let mut verdicts = match outcome {
        crate::validation::data::PreparedCustomInvariantExecutionOutcome::Verdict(
            crate::validation::data::CustomInvariantVerdict::Pass,
        ) => vec![InvariantVerdict::Pass],
        crate::validation::data::PreparedCustomInvariantExecutionOutcome::Verdict(
            crate::validation::data::CustomInvariantVerdict::Violation,
        ) => vec![InvariantVerdict::Violation(
            crate::validation::data::InvariantViolation {
                class: registration.execution_point().class(),
                code: crate::diagnostics::data::DiagnosticCode::InvariantViolation,
                detail: format!(
                    "custom invariant '{}' reported a structural violation",
                    registration.rule_id().as_str()
                ),
                fields:
                    crate::validation::data::InvariantViolationFields::CustomInvariantViolation {
                        identity: registration.descriptor().identity.clone(),
                    },
            },
        )],
        crate::validation::data::PreparedCustomInvariantExecutionOutcome::Failure(failure) => {
            if failure.kind == CustomInvariantFailureKind::Panic
                && failure.phase == CustomInvariantRuntimePhase::Execution
            {
                runtime.performance_access().count_custom_invariant_panic();
            }
            vec![InvariantVerdict::Violation(
                custom_invariant_failure_violation(
                    registration.execution_point().class(),
                    &failure,
                ),
            )]
        }
    };
    let custom_provenance = context.provenance();
    if let (Some(budget), Some(structural)) = (budget, structural_budget.as_ref()) {
        let _ = budget.settle_custom_structural(structural);
    }
    if work.exceeded() {
        verdicts = vec![InvariantVerdict::Violation(
            crate::validation::data::InvariantViolation {
                class: registration.execution_point().class(),
                code: crate::diagnostics::data::DiagnosticCode::InvariantViolation,
                detail: format!(
                    "custom invariant '{}' exceeded its installed work budget",
                    registration.rule_id().as_str(),
                ),
                fields:
                    crate::validation::data::InvariantViolationFields::CustomInvariantViolation {
                        identity: registration.descriptor().identity.clone(),
                    },
            },
        )];
    }
    RegisteredInvariantEvaluation {
        reported_rule: InvariantReportedRule::Custom(registration.descriptor().identity.clone()),
        groups: registration.groups(),
        cost: registration.cost_class(),
        custom_provenance: Some(custom_provenance),
        verdicts,
    }
}

fn custom_invariant_failure_violation(
    class: crate::validation::data::InvariantClass,
    failure: &CustomInvariantFailure,
) -> crate::validation::data::InvariantViolation {
    crate::validation::data::InvariantViolation {
        class,
        code: crate::diagnostics::data::DiagnosticCode::InvariantViolation,
        detail: format!(
            "custom invariant '{}' failed during {}: {}",
            failure.identity.rule_id.as_str(),
            failure.phase.diagnostic_label(),
            failure.detail
        ),
        fields: crate::validation::data::InvariantViolationFields::CustomInvariantFailure {
            identity: crate::validation::data::CustomInvariantFailureIdentity::new(
                failure.identity.clone(),
            ),
            phase: custom_invariant_failure_phase(failure.phase),
            failure: custom_invariant_failure_kind(failure.kind),
            detail: failure.detail.to_string(),
        },
    }
}

fn custom_invariant_failure_phase(
    phase: CustomInvariantRuntimePhase,
) -> crate::validation::data::CustomInvariantFailurePhase {
    match phase {
        CustomInvariantRuntimePhase::Preparation => {
            crate::validation::data::CustomInvariantFailurePhase::Preparation
        }
        CustomInvariantRuntimePhase::Execution => {
            crate::validation::data::CustomInvariantFailurePhase::Execution
        }
    }
}

fn custom_invariant_failure_kind(
    failure: CustomInvariantFailureKind,
) -> crate::validation::data::ResultCustomInvariantFailureKind {
    match failure {
        CustomInvariantFailureKind::PreparationError => {
            crate::validation::data::ResultCustomInvariantFailureKind::PreparationError
        }
        CustomInvariantFailureKind::ExecutionError => {
            crate::validation::data::ResultCustomInvariantFailureKind::ExecutionError
        }
        CustomInvariantFailureKind::Panic => {
            crate::validation::data::ResultCustomInvariantFailureKind::Panic
        }
    }
}
