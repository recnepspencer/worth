//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Query, Parameters, QueryResult, Scope>
    WorthQueryApplicationQueryContinuation<Schema, Query, Parameters, QueryResult, Scope>
{
    #[doc(hidden)]
    pub fn index_generation_for_test(&self) -> &DerivedIndexGenerationId {
        &self.index_generation
    }
}

impl<Schema, Query, Parameters, QueryResult, Scope>
    WorthQueryApplicationQueryContinuation<Schema, Query, Parameters, QueryResult, Scope>
{
    #[doc(hidden)]
    pub fn index_id_for_test(&self) -> &DerivedIndexId {
        &self.index_id
    }
}

impl<Schema, Query, Parameters, QueryResult, Scope>
    WorthQueryApplicationQueryContinuation<Schema, Query, Parameters, QueryResult, Scope>
{
    #[doc(hidden)]
    pub fn product_for_test(&self) -> &crate::basis::WorthQueryProductObservationLease {
        &self.product
    }
}

impl<Schema, Query, Parameters, QueryResult, Scope>
    WorthQueryApplicationQueryContinuation<Schema, Query, Parameters, QueryResult, Scope>
{
    #[doc(hidden)]
    pub fn provider_identity_for_test(&self) -> &String {
        &self.provider_identity
    }
}

impl<Schema, Query, Parameters, QueryResult, Scope>
    WorthQueryApplicationQueryContinuation<Schema, Query, Parameters, QueryResult, Scope>
{
    #[doc(hidden)]
    pub fn scope_entity_id_for_test(&self) -> &EntityId {
        &self.scope_entity_id
    }
}

impl<Schema, Query, Parameters, QueryResult, Scope>
    WorthQueryApplicationQueryContinuation<Schema, Query, Parameters, QueryResult, Scope>
{
    #[doc(hidden)]
    pub fn continuation_contract_digest_mut_for_test(&mut self) -> &mut CanonicalDigestId {
        &mut self.continuation_contract_digest
    }
}

impl<Schema, Query, Parameters, QueryResult, Scope>
    WorthQueryApplicationQueryContinuation<Schema, Query, Parameters, QueryResult, Scope>
{
    #[doc(hidden)]
    pub fn product_mut_for_test(&mut self) -> &mut crate::basis::WorthQueryProductObservationLease {
        &mut self.product
    }
}

impl<Schema, Query, Parameters, QueryResult, Scope>
    WorthQueryApplicationQueryContinuation<Schema, Query, Parameters, QueryResult, Scope>
{
    #[doc(hidden)]
    pub fn product_into_for_test(self) -> crate::basis::WorthQueryProductObservationLease {
        self.product
    }
}

impl<Schema, Query, Parameters, QueryResult, Scope>
    WorthQueryApplicationQueryContinuation<Schema, Query, Parameters, QueryResult, Scope>
{
    #[doc(hidden)]
    pub fn provider_identity_mut_for_test(&mut self) -> &mut String {
        &mut self.provider_identity
    }
}
