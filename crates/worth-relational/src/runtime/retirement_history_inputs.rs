//! Bind reconstruction requests while the receiver still carries its native owners.
use crate::history::data::codec_retirement_closure::history_request::RetirementHistoryInputs;
use crate::runtime::RelationalRuntime;
use std::sync::Arc;

impl<'a> From<&'a RelationalRuntime> for RetirementHistoryInputs<'a> {
    fn from(runtime: &'a RelationalRuntime) -> Self {
        Self {
            plans: &runtime.schema_contract_runtime.aspect_contract_plans,
            projection: runtime.into(),
        }
    }
}

impl<'a> From<&'a mut RelationalRuntime> for RetirementHistoryInputs<'a> {
    fn from(runtime: &'a mut RelationalRuntime) -> Self {
        (&*runtime).into()
    }
}

impl<'a> From<&'a Arc<RelationalRuntime>> for RetirementHistoryInputs<'a> {
    fn from(runtime: &'a Arc<RelationalRuntime>) -> Self {
        runtime.as_ref().into()
    }
}
