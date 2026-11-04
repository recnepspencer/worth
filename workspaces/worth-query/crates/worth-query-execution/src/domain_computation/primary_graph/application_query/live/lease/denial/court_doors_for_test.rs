//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope, Target, Binding,>
    WorthQueryApplicationLiveLease<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Target,
        Binding,
    >
where
    Schema: ApplicationSchema,
    QueryResult: WorthQueryApplicationProjection<Schema, Query>,
    Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
{
    #[doc(hidden)]
    pub fn handle_projection_denial_for_test(
        &mut self,
        kind: WorthQueryApplicationProjectionDenialKind,
    ) -> WorthQueryApplicationLiveOutcome<Query, QueryResult> {
        Self::handle_projection_denial(self, kind)
    }
}
