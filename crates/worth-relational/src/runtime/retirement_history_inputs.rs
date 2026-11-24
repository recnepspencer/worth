//! Bind reconstruction requests while the receiver still carries its native owners.
use crate::history::data::codec_retirement_closure::history_request::RetirementHistoryInputs;
use crate::runtime::RelationalRuntime;

impl<'a> From<&'a RelationalRuntime> for RetirementHistoryInputs<'a> {
    fn from(runtime: &'a RelationalRuntime) -> Self {
        Self {
            plans: &runtime.schema_contract_runtime.aspect_contract_plans,
            projection: runtime.into(),
            canonical_history: runtime.history.canonical_read(),
            deletion_grammar:
                crate::authority::mutation::deletion_publication::codec_expectation::GRAMMAR,
        }
    }
}

impl<'a> From<&'a mut RelationalRuntime> for RetirementHistoryInputs<'a> {
    fn from(runtime: &'a mut RelationalRuntime) -> Self {
        (&*runtime).into()
    }
}
