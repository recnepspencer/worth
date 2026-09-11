use worth_query_decl::facade::application_query::ApplicationQueryLiveCauseBinding;
use worth_query_decl::facade::worth_query_portable_type;

use crate::{
    estate::{EmergencyAccessId, EstateCaseId},
    schema::{
        BankSchema, EmergencyAccess, EmergencyAccessIdBinding, EstateCase, EstateCaseIdBinding,
        EstateEmergencyAccessActivityEffect, EstateEmergencyAccessActivityEvent,
        EstateEmergencyAccessActivityEventBinding,
    },
};

use super::EstateEmergencyAccessActivityQuery;

pub struct EstateEmergencyAccessActivityLiveCause;
worth_query_portable_type!(EstateEmergencyAccessActivityLiveCause => "EstateEmergencyAccessActivityLiveCause");

impl
    ApplicationQueryLiveCauseBinding<
        BankSchema,
        EstateEmergencyAccessActivityQuery,
        EstateCase,
        EmergencyAccess,
    > for EstateEmergencyAccessActivityLiveCause
{
    type Effect = EstateEmergencyAccessActivityEffect;
    type PayloadBinding = EstateEmergencyAccessActivityEventBinding;
    type ScopeIdentityBinding = EstateCaseIdBinding;
    type TargetIdentityBinding = EmergencyAccessIdBinding;

    fn effect() -> worth_query_decl::facade::application_schema::ApplicationEffectRef<
        BankSchema,
        Self::Effect,
        EstateEmergencyAccessActivityEvent,
    > {
        EstateEmergencyAccessActivityEffect::reference()
    }

    fn scope_identity(payload: &EstateEmergencyAccessActivityEvent) -> EstateCaseId {
        payload.estate
    }

    fn target_identity(payload: &EstateEmergencyAccessActivityEvent) -> EmergencyAccessId {
        payload.access
    }
}
