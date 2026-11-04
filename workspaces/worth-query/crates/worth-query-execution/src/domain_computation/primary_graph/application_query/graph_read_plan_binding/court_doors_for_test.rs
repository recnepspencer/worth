//! Sealed test doors over this module's private members.
use super::*;

impl<'a, Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>
    WorthQueryAdmittedApplicationQueryPlan<
        'a,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >
{
    #[doc(hidden)]
    pub fn continuation_index_id_for_test(&self) -> &Option<DerivedIndexId> {
        &self.continuation_index_id
    }
}

impl<'a, Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>
    WorthQueryAdmittedApplicationQueryPlan<
        'a,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >
{
    #[doc(hidden)]
    pub fn governance_for_test(
        &self,
    ) -> &super::super::disclosure::WorthQueryApplicationQueryGovernance {
        &self.governance
    }
}

impl<'a, Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>
    WorthQueryAdmittedApplicationQueryPlan<
        'a,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >
{
    #[doc(hidden)]
    pub fn parameters_for_test(&self) -> &WorthQueryAdmittedApplicationQueryParameters {
        &self.parameters
    }
}

impl<'a, Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>
    WorthQueryAdmittedApplicationQueryPlan<
        'a,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >
{
    #[doc(hidden)]
    pub fn query_for_test(
        &self,
    ) -> &&'a WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>
    {
        &self.query
    }
}

impl<'a, Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>
    WorthQueryAdmittedApplicationQueryPlan<
        'a,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >
{
    #[doc(hidden)]
    pub fn runtime_authority_for_test(&self) -> &WorthQueryRuntimeAuthorityIdentity {
        &self.runtime_authority
    }
}

impl<'a, Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>
    WorthQueryAdmittedApplicationQueryPlan<
        'a,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >
{
    #[doc(hidden)]
    pub fn scope_for_test(&self) -> &&'a WorthQueryApplicationEntityIdentity<Schema, Scope> {
        &self.scope
    }
}

impl<'a, Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>
    WorthQueryAdmittedApplicationQueryPlan<
        'a,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >
{
    #[doc(hidden)]
    pub fn continuation_index_id_mut_for_test(&mut self) -> &mut Option<DerivedIndexId> {
        &mut self.continuation_index_id
    }
}
