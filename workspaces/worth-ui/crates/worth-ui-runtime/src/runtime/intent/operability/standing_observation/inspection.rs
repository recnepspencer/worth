use super::{
    UiIntentStandingDecision, UiIntentStandingOperabilityObservation,
    UiIntentStandingOperabilityUnavailable,
};
use worth_ui_inspection::{
    UiPointerAffordanceInspectionDecision as Decision,
    UiPointerAffordanceInspectionInoperableCause as Cause,
    UiPointerAffordanceInspectionUnavailable as Unavailable,
};

#[path = "inspection/denial.rs"]
mod denial;

impl UiIntentStandingOperabilityObservation {
    /// `expressions` is the catalog of the generation this observation was
    /// made in; withheld conditions are named from it. `None` when it does
    /// not install a condition the decision names, so the observation was
    /// not made against it.
    pub(crate) fn inspection_decision(
        &self,
        expressions: &crate::runtime::expression::UiExpressionCatalog,
    ) -> Option<Decision> {
        if let Some(decision) = self.product_decision() {
            return Some(Decision::Product {
                contract_identity: decision.contract_identity().into(),
                causes: decision
                    .causes()
                    .map(|cause| project_cause(cause, expressions))
                    .collect::<Option<_>>()?,
                selected_dependencies_visited: decision.cost().selected_dependencies_visited(),
            });
        }
        let UiIntentStandingDecision::Confirmation(observation) = &self.decision else {
            unreachable!("product decision was projected above")
        };
        Some(Decision::Confirmation {
            eligible: observation.is_eligible(),
            stop: observation.stop_reason().map(denial::confirmation),
            stop_detail: observation
                .stop_reason()
                .map(|stop| format!("{stop:?}").into()),
            expiry_wake_millis: observation.expiry_wake_millis(),
            slots_inspected: observation.cost().slots_inspected(),
        })
    }
}

impl UiIntentStandingOperabilityUnavailable {
    pub(crate) fn inspection(&self) -> Unavailable {
        match self {
            Self::Target(cause) => Unavailable::Target {
                cause: denial::target(cause),
                detail: format!("{cause:?}").into(),
            },
            Self::Presentation(cause) => Unavailable::Presentation {
                cause: denial::presentation(*cause),
                detail: format!("{cause:?}").into(),
            },
            Self::MissingActivationRoute => Unavailable::MissingActivationRoute,
            Self::ConfirmationTimeUnavailable => Unavailable::ConfirmationTimeUnavailable,
        }
    }
}

fn project_cause(
    cause: crate::runtime::intent::UiIntentInoperableCause,
    expressions: &crate::runtime::expression::UiExpressionCatalog,
) -> Option<Cause> {
    use crate::runtime::intent::UiIntentInoperableCause as Owner;
    Some(match cause {
        Owner::Unsupported => Cause::Unsupported,
        Owner::WrongWorld => Cause::WrongWorld,
        Owner::RebindRequired => Cause::RebindRequired,
        Owner::StaleTarget => Cause::StaleTarget,
        Owner::ConditionWithheld { axis, condition } => Cause::ConditionWithheld {
            axis: denial::condition_axis(axis),
            condition_identity: expressions.expression(condition.slot())?.identity().into(),
            withholding: denial::condition_withholding(condition.withholding()),
        },
        Owner::PolicyDenied => Cause::PolicyDenied,
        Owner::Occupied => Cause::Occupied,
        Owner::Readonly => Cause::Readonly,
        Owner::Pending => Cause::Pending,
        Owner::ConfirmationRequired { policy_identity } => {
            Cause::ConfirmationRequired { policy_identity }
        }
    })
}
