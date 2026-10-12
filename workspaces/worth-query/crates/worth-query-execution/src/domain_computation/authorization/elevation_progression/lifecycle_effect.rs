use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use std::any::Any;

use worth_query_declaration::facade::application_capability::ApplicationCapabilityTransitionBinding;
use worth_query_declaration::lifecycle_effect_derivation_authority::{
    derive_application_capability_lifecycle_effect, DerivedApplicationCapabilityLifecycleEffect,
};

use super::super::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

pub(super) fn derive_lifecycle_effect<Input: 'static>(
    transition: &ApplicationCapabilityTransitionBinding,
    input: &Input,
    subject: &str,
) -> Result<
    Option<DerivedApplicationCapabilityLifecycleEffect>,
    WorthQueryOperationAuthorizationDenial,
> {
    transition
        .lifecycle_effect()
        .map(|binding| {
            derive_application_capability_lifecycle_effect(binding, input as &dyn Any).ok_or_else(
                || {
                    WorthQueryOperationAuthorizationDenial::new(
                        WorthQueryOperationAuthorizationDenialKind::Refused(
                            AuthorizationRefusal::InconsistentDecision,
                        ),
                        subject,
                    )
                },
            )
        })
        .transpose()
}
