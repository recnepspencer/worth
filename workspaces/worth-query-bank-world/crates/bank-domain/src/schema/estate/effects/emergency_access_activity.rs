use worth_query_decl::facade::worth_query_structured_value_binding;
use worth_query_decl::facade::{
    application_capability::ApplicationCapabilityLifecycleEffect,
    application_schema::{ApplicationEffectRef, ApplicationRetainedEffectBinding},
    worth_query_effect,
};

use crate::{
    estate::{EmergencyAccessId, EstateAction, EstateCaseId},
    schema::{
        ApproveEstateEmergencyAccessOperation, BankSchema, CompleteEstateMandatoryReviewOperation,
        RequestEstateEmergencyAccessOperation, RevokeEstateEmergencyAccessOperation,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateEmergencyAccessActivityEvent {
    pub estate: EstateCaseId,
    pub access: EmergencyAccessId,
}

worth_query_structured_value_binding!(pub EstateEmergencyAccessActivityEventBinding for EstateEmergencyAccessActivityEvent { identity: "bank.estate.effect.emergency-access-activity.payload.v1" });
impl ApplicationRetainedEffectBinding for EstateEmergencyAccessActivityEventBinding {
    fn retained_bytes(_: &Self::Value) -> u64 {
        u64::try_from(std::mem::size_of::<EstateEmergencyAccessActivityEvent>()).unwrap_or(u64::MAX)
    }
}
worth_query_effect!(pub EstateEmergencyAccessActivityEffect for BankSchema, payload EstateEmergencyAccessActivityEventBinding);

macro_rules! lifecycle_effect {
    ($operation:ty, $variant:pat => ($estate:expr, $access:expr)) => {
        impl ApplicationCapabilityLifecycleEffect<BankSchema, $operation> for EstateAction {
            type Effect = EstateEmergencyAccessActivityEffect;
            type PayloadBinding = EstateEmergencyAccessActivityEventBinding;

            fn effect(
            ) -> ApplicationEffectRef<BankSchema, Self::Effect, EstateEmergencyAccessActivityEvent>
            {
                EstateEmergencyAccessActivityEffect::reference()
            }

            fn lifecycle_effect(&self) -> Option<EstateEmergencyAccessActivityEvent> {
                let $variant = *self else {
                    return None;
                };
                Some(EstateEmergencyAccessActivityEvent {
                    estate: $estate,
                    access: $access,
                })
            }
        }
    };
}

lifecycle_effect!(
    RequestEstateEmergencyAccessOperation,
    EstateAction::RequestEmergencyAccess { estate, access, .. } => (estate, access)
);
lifecycle_effect!(
    ApproveEstateEmergencyAccessOperation,
    EstateAction::ApproveEmergencyAccess { estate, access } => (estate, access)
);
lifecycle_effect!(
    RevokeEstateEmergencyAccessOperation,
    EstateAction::RevokeEmergencyAccess { estate, access } => (estate, access)
);
lifecycle_effect!(
    CompleteEstateMandatoryReviewOperation,
    EstateAction::CompleteMandatoryReview { estate, access, .. } => (estate, access)
);

#[cfg(test)]
mod tests {
    use worth_query_decl::facade::application_capability::ApplicationCapabilityLifecycleEffect;
    use worth_query_decl::facade::application_schema::ApplicationRetainedEffectBinding;

    use super::{EstateEmergencyAccessActivityEvent, EstateEmergencyAccessActivityEventBinding};
    use crate::{
        estate::{EmergencyAccessId, EstateAction, EstateCaseId},
        schema::{BankSchema, RequestEstateEmergencyAccessOperation},
    };

    #[test]
    fn activity_event_retains_exact_fixed_width() {
        let event = EstateEmergencyAccessActivityEvent {
            estate: EstateCaseId::new(1).unwrap(),
            access: EmergencyAccessId::new(2).unwrap(),
        };

        assert_eq!(
            EstateEmergencyAccessActivityEventBinding::retained_bytes(&event),
            u64::try_from(std::mem::size_of::<EstateEmergencyAccessActivityEvent>()).unwrap()
        );
    }

    #[test]
    fn wrong_operation_variant_derives_no_activity_effect() {
        let action = EstateAction::ApproveEmergencyAccess {
            estate: EstateCaseId::new(1).unwrap(),
            access: EmergencyAccessId::new(2).unwrap(),
        };

        let payload = <EstateAction as ApplicationCapabilityLifecycleEffect<
            BankSchema,
            RequestEstateEmergencyAccessOperation,
        >>::lifecycle_effect(&action);

        assert_eq!(payload, None);
    }
}
