use worth_query_decl::facade::{
    application_query::ApplicationQueryLiveCauseBinding,
    application_schema::{
        ApplicationEffectRef, ApplicationRetainedEffectBinding, StringApplicationValueBinding,
    },
    worth_query_effect, worth_query_portable_type, worth_query_structured_value_binding,
};

use super::{WorthUiApplicationSchema, WorthUiRecord, WorthUiStatusQuery};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiStatusChanged {
    pub identity: String,
}

worth_query_structured_value_binding!(pub WorthUiStatusChangedBinding for WorthUiStatusChanged {
    identity: "worth.ui.status-changed-payload.v1"
});

impl ApplicationRetainedEffectBinding for WorthUiStatusChangedBinding {
    fn retained_bytes(value: &Self::Value) -> u64 {
        u64::try_from(value.identity.len()).unwrap_or(u64::MAX)
    }
}

worth_query_effect!(pub WorthUiStatusChangedEffect for WorthUiApplicationSchema,
    payload WorthUiStatusChangedBinding);

pub struct WorthUiStatusLiveCause;
worth_query_portable_type!(WorthUiStatusLiveCause => "worth.ui.status-live-cause.v1");

impl
    ApplicationQueryLiveCauseBinding<
        WorthUiApplicationSchema,
        WorthUiStatusQuery,
        WorthUiRecord,
        WorthUiRecord,
    > for WorthUiStatusLiveCause
{
    type Effect = WorthUiStatusChangedEffect;
    type PayloadBinding = WorthUiStatusChangedBinding;
    type ScopeIdentityBinding = StringApplicationValueBinding;
    type TargetIdentityBinding = StringApplicationValueBinding;

    fn effect() -> ApplicationEffectRef<WorthUiApplicationSchema, Self::Effect, WorthUiStatusChanged>
    {
        WorthUiStatusChangedEffect::reference()
    }

    fn scope_identity(payload: &WorthUiStatusChanged) -> String {
        payload.identity.clone()
    }

    fn target_identity(payload: &WorthUiStatusChanged) -> String {
        payload.identity.clone()
    }
}
