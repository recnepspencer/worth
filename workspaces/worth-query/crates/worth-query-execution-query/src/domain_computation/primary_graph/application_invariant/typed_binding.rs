use std::marker::PhantomData;

use worth_foundational::facade::AspectFieldLocator;
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;
use worth_relational::facade::identity::KindId;

/// Installed entity-kind access for one exact application schema binding.
pub struct WorthQueryApplicationInvariantEntityBinding<Schema, Entity> {
    pub(crate) binding_identity:
        ApplicationSchemaBindingIdentity,
    pub(crate) entity_kind: KindId,
    pub(crate) _marker: PhantomData<fn() -> (Schema, Entity)>,
}

/// Exact installed recipe with complete physical read/applicability coverage.
pub struct WorthQueryApplicationInvariantDerivedRelationBinding<Schema, Relation, From, To> {
    pub(crate) binding_identity:
        ApplicationSchemaBindingIdentity,
    pub(crate) definition:
        worth_relational::facade::schema::DerivedRelationHandle,
    pub(crate) from:
        super::super::schema_layout::MemberDomainBinding,
    pub(crate) to:
        super::super::schema_layout::MemberDomainBinding,
    pub(crate) orbit_definition:
        Option<worth_foundational::facade::AggregateDefinitionIdentity>,
    pub(crate) _marker:
        PhantomData<fn() -> (Schema, Relation, From, To)>,
}

/// An actual installed domain case, with complete carrier access coverage.
pub struct WorthQueryApplicationInvariantProjectedCaseBinding<Schema, Domain, Carrier, Case> {
    pub(crate) binding_identity:
        ApplicationSchemaBindingIdentity,
    pub(crate) case:
        worth_relational::facade::schema::ProjectedMemberCaseHandle,
    pub(crate) _marker:
        PhantomData<fn() -> (Schema, Domain, Carrier, Case)>,
}

/// Installed component definition with complete declared read and applicability coverage.
pub struct WorthQueryApplicationInvariantComponentSetBinding<Schema, Set, Entity> {
    pub(crate) binding_identity:
        ApplicationSchemaBindingIdentity,
    pub(crate) definition:
        worth_foundational::facade::AggregateDefinitionIdentity,
    pub(crate) member_kind: KindId,
    pub(crate) member_domain:
        super::super::schema_layout::MemberDomainBinding,
    pub(crate) source:
        Option<worth_relational::facade::schema::DerivedRelationHandle>,
    pub(crate) sources:
        std::sync::Arc<[(std::sync::Arc<str>, KindId)]>,
    pub(crate) _marker:
        PhantomData<fn() -> (Schema, Set, Entity)>,
}

/// Installed field access for one exact application schema binding.
pub struct WorthQueryApplicationInvariantFieldBinding<Schema, Entity, Value> {
    pub(crate) binding_identity:
        ApplicationSchemaBindingIdentity,
    pub(crate) entity_kind: KindId,
    pub(crate) locator: AspectFieldLocator,
    pub(crate) aspect_absence:
        worth_foundational::facade::AbsenceLaw,
    pub(crate) presence:
        worth_query_declaration::facade::application_schema::ApplicationFieldPresence,
    pub(crate) decode:
        fn(&worth_foundational::facade::AspectValue) -> Result<Value, String>,
    pub(crate) _marker: PhantomData<fn() -> (Schema, Entity)>,
}

/// Installed relation access for one exact application schema binding.
pub struct WorthQueryApplicationInvariantRelationBinding<Schema, Relation, From, To> {
    pub(crate) binding_identity:
        ApplicationSchemaBindingIdentity,
    pub(crate) relation_kind: KindId,
    pub(crate) from_kind: KindId,
    pub(crate) to_kind: KindId,
    pub(crate) orbit: bool,
    pub(crate) _marker:
        PhantomData<fn() -> (Schema, Relation, From, To)>,
}
