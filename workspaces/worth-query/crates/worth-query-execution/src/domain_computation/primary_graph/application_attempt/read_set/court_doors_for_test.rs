//! Sealed test doors over this module's private members.
use super::*;

impl<'request, 'authority, Schema, Operation, Input, Scope, Phase>
    WorthQueryApplicationReadAttempt<'request, 'authority, Schema, Operation, Input, Scope, Phase>
{
    #[doc(hidden)]
    pub fn admission_for_test(
        &self,
    ) -> &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope> {
        &self.admission
    }
}

impl<'request, 'authority, Schema, Operation, Input, Scope, Phase>
    WorthQueryApplicationReadAttempt<'request, 'authority, Schema, Operation, Input, Scope, Phase>
{
    #[doc(hidden)]
    pub fn expected_facts_for_test(
        &self,
    ) -> &Option<crate::domain_computation::primary_graph::invariant_projection::ExpectedFactKeys>
    {
        &self.expected_facts
    }
}

impl<Schema, Operation, Input, Scope, Phase>
    WorthQueryCompleteApplicationReadSet<Schema, Operation, Input, Scope, Phase>
{
    #[doc(hidden)]
    pub fn facts_for_test(&self) -> &FactPayload {
        &self.facts
    }
}

impl<'request, 'authority, Schema, Operation, Input, Scope, Phase>
    WorthQueryApplicationReadAttempt<'request, 'authority, Schema, Operation, Input, Scope, Phase>
{
    #[doc(hidden)]
    pub fn lease_for_test(&self) -> &WorthQueryApplicationSnapshotLease {
        &self.lease
    }
}

impl<'request, 'authority, Schema, Operation, Input, Scope, Phase>
    WorthQueryApplicationReadAttempt<'request, 'authority, Schema, Operation, Input, Scope, Phase>
{
    #[doc(hidden)]
    pub fn read_scope_for_test(&self) -> &WorthQueryApplicationReadScope {
        &self.read_scope
    }
}
