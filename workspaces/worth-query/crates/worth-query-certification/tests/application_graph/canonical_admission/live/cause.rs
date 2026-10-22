//! A real retained effect selects the installed live root.
use super::super::schema::*;
use super::query::LiveQuery;
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    *,
};

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Changed {
    pub identity: String,
}
worth_query_structured_value_binding!(pub ChangedBinding for Changed { identity: "canonical-admission.live-changed.v1" });
worth_query_effect!(pub ValueChanged for AdmissionSchema, payload ChangedBinding);

pub struct LiveCause;
worth_query_portable_type!(LiveCause => "canonical-admission.live-cause.v1");
impl ApplicationQueryLiveCauseBinding<AdmissionSchema, LiveQuery, Document, Document>
    for LiveCause
{
    type Effect = ValueChanged;
    type PayloadBinding = ChangedBinding;
    type ScopeIdentityBinding = StringApplicationValueBinding;
    type TargetIdentityBinding = StringApplicationValueBinding;

    fn effect() -> ApplicationEffectRef<AdmissionSchema, ValueChanged, Changed> {
        ValueChanged::reference()
    }
    fn scope_identity(payload: &Changed) -> String {
        payload.identity.clone()
    }
    fn target_identity(payload: &Changed) -> String {
        payload.identity.clone()
    }
}
