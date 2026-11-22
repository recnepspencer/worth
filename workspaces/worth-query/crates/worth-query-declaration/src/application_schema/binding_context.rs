//! Principal, idempotency and denial declarations shared by a family of bindings.
use super::{
    ApplicationIdentityScalarValueBinding, ApplicationPrincipalBindingRef, ApplicationSchema,
    ApplicationStructuredValueBinding,
};

/// The clauses a schema's mutation and query bindings would otherwise repeat.
///
/// `worth_query_mutation_binding!` and `worth_query_query_binding!` accept
/// `context Context,` in place of the principal group (`principal`,
/// `mapping`, `principal_entity`, `principal_identity`, `identity_binding`)
/// and, for mutations, the `idempotency .., identity ..` and `denial`
/// clauses. A binding that states one of those clauses itself keeps its own
/// value; omitted clauses resolve to this context. The resolved associated
/// types and identities are exactly the ones a fully written binding names,
/// so both spellings install the same canonical schema.
///
/// The context is declarative meaning only. Authentication, idempotency and
/// denial handling stay with their installed owners.
pub trait ApplicationBindingContext<Schema: ApplicationSchema>: 'static {
    type PrincipalBinding: 'static;
    type Mapping: 'static;
    type Principal: 'static;
    type PrincipalIdentity: Clone + 'static;
    type PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<
        Value = Self::PrincipalIdentity,
    >;
    /// Everything the key serializes takes part in the canonical key identity.
    type IdempotencyKey: serde::Serialize + 'static;
    type DenialBinding: ApplicationStructuredValueBinding;

    const IDEMPOTENCY_IDENTITY: &'static str;

    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        Self::PrincipalBinding,
        Self::Mapping,
        Self::Principal,
        Self::PrincipalIdentity,
        Self::PrincipalIdentityBinding,
    >;
}
