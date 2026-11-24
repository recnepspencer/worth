//! Native and custom invariant registration execution.

use crate::contracts::validation::data::catalog::InvariantRegistrationOwnerPort as _;
use crate::contracts::validation::data::custom_rule::errors::{
    CustomInvariantExecutionError, CustomInvariantFailure, CustomInvariantFailureKind,
    CustomInvariantRuntimePhase, CustomInvariantVerdict,
    PreparedCustomInvariantExecutionOutcome as Outcome,
};
use crate::contracts::validation::data::execution::{InvariantReportedRule, InvariantVerdict};
use crate::validation::custom_rule::CustomInvariantExecutionContext;
use crate::validation::custom_rule::CustomInvariantProvenance;
use crate::validation::engine::budget::InvariantBudget;
use crate::validation::engine::context::InvariantExecutionContext;
use crate::validation::engine::evaluator::evaluate_rule;
use crate::validation::engine::InvariantRuntimeView;

use super::super::packets::InvariantWorkPacket;

pub(super) struct RegisteredInvariantEvaluation {
    pub(super) reported_rule: InvariantReportedRule,
    pub(super) groups: crate::contracts::validation::data::groups::InvariantGroupSet,
    pub(super) cost: crate::contracts::validation::data::groups::InvariantCostClass,
    pub(super) custom_provenance: Option<CustomInvariantProvenance>,
    pub(super) verdicts: Vec<InvariantVerdict>,
}

pub(super) fn evaluate_registered_rule<'state>(
    runtime: &InvariantRuntimeView<'state>,
    execution: worth_execution::ExecutionRequest<'state, 'state>,
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
            execution,
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
            retained_touched,
            work,
            basis,
        } => {
            let provenance = CustomInvariantExecutionContext::not_applicable_provenance(
                runtime,
                execution,
                packet.observation,
                packet.version_id,
                packet.current_version_id,
                prepared_scope,
                std::sync::Arc::clone(retained_touched),
                work.clone(),
                std::sync::Arc::new(registration.access_contract().clone()),
                *basis,
            );
            RegisteredInvariantEvaluation {
                reported_rule: InvariantReportedRule::Custom(registration.descriptor().identity.clone()),
                groups: registration.groups(),
                cost: registration.cost_class(),
                custom_provenance: Some(provenance),
                verdicts: vec![InvariantVerdict::NotApplicable],
            }
        }
    }
}

fn evaluate_native_registration<'state>(
    runtime: &InvariantRuntimeView<'state>,
    packet: &InvariantWorkPacket<'state>,
    registration: &crate::contracts::validation::data::catalog::InvariantRegistration,
    budget: Option<&dyn InvariantBudget>,
) -> RegisteredInvariantEvaluation {
    let mut context = InvariantExecutionContext::from_semantic_plan(
        runtime,
        packet.observation.clone(),
        packet.version_id,
        packet.current_version_id,
        packet.semantic_plan,
        packet.relation_integrity_scopes.clone(),
        std::sync::Arc::clone(&packet.current_version_minimum_index),
    );
    if let Some(budget) = budget {
        context = context.with_budget(budget);
    }
    if let crate::contracts::validation::data::rules::InvariantRule::EndpointKindContract(
        contract,
    ) = &registration.rule
    {
        context = context.with_created_endpoint_scope(contract.relation_kind_id);
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
    execution: worth_execution::ExecutionRequest<'state, 'state>,
    packet: &InvariantWorkPacket<'state>,
    registration: &crate::validation::custom_rule::CustomInvariantRegistration,
    prepared_execution: &std::sync::Arc<
        dyn crate::validation::custom_rule::PreparedCustomInvariantExecution,
    >,
    prepared_scope: &crate::validation::custom_rule::PreparedCustomInvariantScope,
    retained_touched: &std::sync::Arc<
        crate::contracts::validation::data::custom_rule::touched_scope::TouchedStructuralSet,
    >,
    budget: Option<&dyn InvariantBudget>,
) -> RegisteredInvariantEvaluation {
    let structural_budget = budget.and_then(InvariantBudget::custom_structural_budget);
    let work = prepared_execution.work_meter();
    let work = structural_budget
        .as_ref()
        .map_or(work.clone(), |structural| {
            if !registration.supports_checked_preparation()
                && runtime.custom_kernel_posture == worth_foundational::ExecutionPosture::Serial
            {
                work.clone().with_lease(structural)
            } else {
                work.for_evaluation(structural)
            }
        });
    let ready = budget.is_none_or(|budget| budget.checkpoint(1));
    let context = CustomInvariantExecutionContext::new(
        runtime,
        execution,
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
        Outcome::Verdict(CustomInvariantVerdict::Pass)
    } else if let Some(budget) = budget {
        match prepared_execution.evaluate_checked(&context, budget.custom_meter()) {
            Some(outcome) => outcome,
            None if runtime.custom_kernel_posture
                == worth_foundational::ExecutionPosture::Serial =>
            {
                prepared_execution.evaluate(&context)
            }
            None => {
                budget.deny_unchecked_custom();
                Outcome::Verdict(CustomInvariantVerdict::Pass)
            }
        }
    } else {
        prepared_execution.evaluate(&context)
    };
    let mut verdicts = match outcome {
        Outcome::Verdict(
            CustomInvariantVerdict::Pass,
        ) => vec![InvariantVerdict::Pass],
        Outcome::Verdict(
            CustomInvariantVerdict::Violation(refusal),
        ) => vec![InvariantVerdict::Violation(
            crate::contracts::validation::data::results::InvariantViolation {
                class: registration.execution_point().class(),
                code: crate::contracts::diagnostics::data::artifacts::DiagnosticCode::InvariantViolation,
                detail: format!(
                    "custom invariant '{}' reported a structural violation",
                    registration.rule_id().as_str()
                ),
                fields:
                    crate::contracts::validation::data::results::InvariantViolationFields::CustomInvariantViolation {
                        identity: registration.descriptor().identity.clone(),
                        refusal,
                    },
            },
        )],
        Outcome::Failure(failure) => {
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
            custom_invariant_failure_violation(
                registration.execution_point().class(),
                &CustomInvariantFailure::execution_error(
                    &registration.descriptor().identity,
                    CustomInvariantExecutionError::new(
                        "custom invariant exceeded its installed work budget",
                    ),
                ),
            ),
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
    class: crate::contracts::validation::data::execution::InvariantClass,
    failure: &CustomInvariantFailure,
) -> crate::contracts::validation::data::results::InvariantViolation {
    crate::contracts::validation::data::results::InvariantViolation {
        class,
        code: crate::contracts::diagnostics::data::artifacts::DiagnosticCode::InvariantViolation,
        detail: format!(
            "custom invariant '{}' failed during {}: {}",
            failure.identity.rule_id.as_str(),
            failure.phase.diagnostic_label(),
            failure.detail
        ),
        fields:
            crate::contracts::validation::data::results::InvariantViolationFields::CustomInvariantFailure {
                identity: crate::contracts::validation::data::results::custom_invariant_failure::CustomInvariantFailureIdentity::new(
                    failure.identity.clone(),
                ),
                phase: custom_invariant_failure_phase(failure.phase),
                failure: custom_invariant_failure_kind(failure.kind),
                cause: failure.cause.clone(),
                detail: failure.detail.to_string(),
            },
    }
}

fn custom_invariant_failure_phase(
    phase: CustomInvariantRuntimePhase,
) -> crate::contracts::validation::data::results::custom_invariant_failure::CustomInvariantFailurePhase{
    match phase {
        CustomInvariantRuntimePhase::Preparation => {
            crate::contracts::validation::data::results::custom_invariant_failure::CustomInvariantFailurePhase::Preparation
        }
        CustomInvariantRuntimePhase::Execution => {
            crate::contracts::validation::data::results::custom_invariant_failure::CustomInvariantFailurePhase::Execution
        }
    }
}

fn custom_invariant_failure_kind(
    failure: CustomInvariantFailureKind,
) -> crate::contracts::validation::data::results::custom_invariant_failure::CustomInvariantFailureKind
{
    match failure {
        CustomInvariantFailureKind::PreparationError => {
            crate::contracts::validation::data::results::custom_invariant_failure::CustomInvariantFailureKind::PreparationError
        }
        CustomInvariantFailureKind::ExecutionError => {
            crate::contracts::validation::data::results::custom_invariant_failure::CustomInvariantFailureKind::ExecutionError
        }
        CustomInvariantFailureKind::Panic => {
            crate::contracts::validation::data::results::custom_invariant_failure::CustomInvariantFailureKind::Panic
        }
    }
}
