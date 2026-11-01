//! Separate live navigation grants are outside the selected model record.
use super::*;
use worth_query_decl::facade::{worth_query_aspect, worth_query_entity, worth_query_field};
worth_query_entity!(pub PlanarNavigationGrant for Schema: TopologySchemaBinding);
worth_query_aspect!(pub PlanarNavigationGrantFacts for Schema: TopologySchemaBinding, PlanarNavigationGrant;
    identity = AspectIdentity(0x9182_0301), revision = AspectContractRevision(1),);
macro_rules! string_field { ($($name:ident),+) => {$(worth_query_field!(pub $name for Schema: TopologySchemaBinding,
    PlanarNavigationGrant, PlanarNavigationGrantFacts: String => StringApplicationValueBinding, read_write, equality);)+}; }
string_field!(
    PlanarNavigationGrantIdentity,
    PlanarNavigationScopeKey,
    PlanarNavigationAction,
    PlanarNavigationPurpose,
    PlanarNavigationStatus
);
macro_rules! number_field { ($($name:ident),+) => {$(worth_query_field!(pub $name for Schema: TopologySchemaBinding,
    PlanarNavigationGrant, PlanarNavigationGrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);)+}; }
number_field!(
    PlanarNavigationNotBefore,
    PlanarNavigationNotAfter,
    PlanarNavigationDepth
);
macro_rules! relation {
    ($name:ident, $from:ident, $to:ident) => {
        pub struct $name;
        impl $name {
            pub fn reference<S: TopologySchemaBinding>(
            ) -> ApplicationRelationRef<S, Self, $from, $to> {
                ApplicationRelationRef::from_schema_identifiers(
                    stringify!($name),
                    stringify!($from),
                    stringify!($to),
                    ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
                )
            }
        }
    };
}
relation!(PlanarNavigationResource, PlanarNavigationGrant, Body);
relation!(
    PlanarNavigationParent,
    PlanarNavigationGrant,
    PlanarNavigationGrant
);
relation!(PlanarNavigationGrantor, Principal, PlanarNavigationGrant);
relation!(PlanarNavigationGrantee, Principal, PlanarNavigationGrant);

pub(super) fn declare<S: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    schema
        .entity(PlanarNavigationGrant::reference())
        .aspect(
            PlanarNavigationGrant::reference(),
            PlanarNavigationGrantFacts::reference(),
        )
        .unique_field(
            PlanarNavigationGrant::reference(),
            PlanarNavigationGrantIdentity::reference(),
        )
        .field(
            PlanarNavigationGrant::reference(),
            PlanarNavigationScopeKey::reference(),
        )
        .field(
            PlanarNavigationGrant::reference(),
            PlanarNavigationAction::reference(),
        )
        .field(
            PlanarNavigationGrant::reference(),
            PlanarNavigationPurpose::reference(),
        )
        .field(
            PlanarNavigationGrant::reference(),
            PlanarNavigationStatus::reference(),
        )
        .field(
            PlanarNavigationGrant::reference(),
            PlanarNavigationNotBefore::reference(),
        )
        .field(
            PlanarNavigationGrant::reference(),
            PlanarNavigationNotAfter::reference(),
        )
        .field(
            PlanarNavigationGrant::reference(),
            PlanarNavigationDepth::reference(),
        )
        .relation(
            PlanarNavigationResource::reference(),
            PlanarNavigationGrant::reference(),
            Body::reference(),
        )
        .relation(
            PlanarNavigationParent::reference(),
            PlanarNavigationGrant::reference(),
            PlanarNavigationGrant::reference(),
        )
        .relation(
            PlanarNavigationGrantor::reference(),
            Principal::reference(),
            PlanarNavigationGrant::reference(),
        )
        .relation(
            PlanarNavigationGrantee::reference(),
            Principal::reference(),
            PlanarNavigationGrant::reference(),
        )
}
