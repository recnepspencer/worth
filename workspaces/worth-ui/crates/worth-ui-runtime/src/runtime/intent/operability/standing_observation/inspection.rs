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
    pub(crate) fn inspection_decision(&self) -> Decision {
        if let Some(decision) = self.product_decision() {
            return Decision::Product {
                contract_identity: decision.contract_identity().into(),
                causes: decision.causes().map(project_cause).collect(),
                selected_dependencies_visited: decision.cost().selected_dependencies_visited(),
            };
        }
        let UiIntentStandingDecision::Confirmation(observation) = &self.decision else {
            unreachable!("product decision was projected above")
        };
        Decision::Confirmation {
            eligible: observation.is_eligible(),
            stop: observation.stop_reason().map(denial::confirmation),
            stop_detail: observation
                .stop_reason()
                .map(|stop| format!("{stop:?}").into()),
            expiry_wake_millis: observation.expiry_wake_millis(),
            slots_inspected: observation.cost().slots_inspected(),
        }
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

fn project_cause(cause: crate::runtime::intent::UiIntentInoperableCause) -> Cause {
    use crate::runtime::intent::UiIntentInoperableCause as Owner;
    match cause {
        Owner::Unsupported => Cause::Unsupported,
        Owner::WrongWorld => Cause::WrongWorld,
        Owner::RebindRequired => Cause::RebindRequired,
        Owner::StaleTarget => Cause::StaleTarget,
        Owner::PolicyDenied => Cause::PolicyDenied,
        Owner::Occupied => Cause::Occupied,
        Owner::Readonly => Cause::Readonly,
        Owner::Pending => Cause::Pending,
        Owner::ConfirmationRequired { policy_identity } => {
            Cause::ConfirmationRequired { policy_identity }
        }
    }
}
