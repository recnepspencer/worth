use std::marker::PhantomData;

use worth_foundational::facade::CanonicalDigestId;
use worth_query_admission::facade::application_query::WorthQueryApplicationParameterCanonicalArtifact;
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledApplicationQueryIdentity,
};
use worth_relational::facade::{
    identity::EntityId,
    indexes::{DerivedIndexGenerationId, DerivedIndexId, RelatedEntityOrderingBoundary},
};

/// Opaque, move-only description of the exact next application-query page.
///
/// This value retains the exact selected World product needed for owner
/// readmission. It retains no authorization or provider session. Resuming
/// consumes it and requires fresh principal, scope, parameter, controls, and
/// installed-query admission.
///
/// It cannot be copied or cloned:
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationQueryContinuation;
///
/// fn clone_continuation<S, Q, P, R, Scope>(
///     continuation: WorthQueryApplicationQueryContinuation<S, Q, P, R, Scope>,
/// ) {
///     let _second = continuation.clone();
/// }
/// ```
///
/// Its private representation cannot be reconstructed from result values:
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationQueryContinuation;
///
/// let _forged = WorthQueryApplicationQueryContinuation::<(), (), (), (), ()> {};
/// ```
///
/// A continuation for one query cannot be substituted for another query:
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::
///     WorthQueryApplicationQueryContinuation;
///
/// fn cross_query<Schema, FirstQuery, SecondQuery, Parameters, Result, Scope>(
///     continuation: WorthQueryApplicationQueryContinuation<
///         Schema, FirstQuery, Parameters, Result, Scope,
///     >,
/// ) -> WorthQueryApplicationQueryContinuation<
///     Schema, SecondQuery, Parameters, Result, Scope,
/// > {
///     continuation
/// }
/// ```
///
/// Nor can a continuation cross its typed scope or result-shape authority:
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::
///     WorthQueryApplicationQueryContinuation;
///
/// fn cross_shape<Schema, Query, Parameters, FirstResult, SecondResult, FirstScope, SecondScope>(
///     continuation: WorthQueryApplicationQueryContinuation<
///         Schema, Query, Parameters, FirstResult, FirstScope,
///     >,
/// ) -> WorthQueryApplicationQueryContinuation<
///     Schema, Query, Parameters, SecondResult, SecondScope,
/// > {
///     continuation
/// }
/// ```
pub struct WorthQueryApplicationQueryContinuation<Schema, Query, Parameters, QueryResult, Scope> {
    pub(super) runtime_authority: u64,
    pub(super) schema_binding: ApplicationSchemaBindingIdentity,
    pub(super) query_identity: WorthQueryInstalledApplicationQueryIdentity,
    pub(super) query_authority_identity: String,
    pub(super) parameter_basis: WorthQueryApplicationParameterCanonicalArtifact,
    pub(super) scope_entity_id: EntityId,
    pub(super) continuation_contract_digest: CanonicalDigestId,
    pub(super) graph_authority_identity: String,
    pub(super) provider_identity: String,
    pub(super) product: crate::basis::WorthQueryProductObservationLease,
    pub(super) index_id: DerivedIndexId,
    pub(super) index_generation: DerivedIndexGenerationId,
    pub(super) boundary: RelatedEntityOrderingBoundary,
    pub(super) page_ordinal: u64,
    pub(super) _marker: PhantomData<fn(Parameters) -> (Schema, Query, QueryResult, Scope)>,
}

impl<Schema, Query, Parameters, QueryResult, Scope>
    WorthQueryApplicationQueryContinuation<Schema, Query, Parameters, QueryResult, Scope>
{
    pub const fn page_ordinal(&self) -> u64 {
        self.page_ordinal
    }
}
