//! Separate live navigation grants are outside the selected model record.
use super::*;
use worth_query_decl::facade::{worth_query_aspect, worth_query_entity, worth_query_field};
worth_query_entity!(pub PlanarHistoryGrant for Schema: TopologySchemaBinding);
worth_query_aspect!(pub PlanarHistoryGrantFacts for Schema: TopologySchemaBinding, PlanarHistoryGrant;
    identity = AspectIdentity(0x9182_0301), revision = AspectContractRevision(1),);
macro_rules! string_field { ($($name:ident),+) => {$(worth_query_field!(pub $name for Schema: TopologySchemaBinding,
    PlanarHistoryGrant, PlanarHistoryGrantFacts: String => StringApplicationValueBinding, read_write, equality);)+}; }
string_field!(
    PlanarHistoryGrantIdentity,
    PlanarHistoryScopeKey,
    PlanarHistoryAction,
    PlanarHistoryPurpose,
    PlanarHistoryStatus
);
macro_rules! number_field { ($($name:ident),+) => {$(worth_query_field!(pub $name for Schema: TopologySchemaBinding,
    PlanarHistoryGrant, PlanarHistoryGrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);)+}; }
number_field!(
    PlanarHistoryNotBefore,
    PlanarHistoryNotAfter,
    PlanarHistoryDepth
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
relation!(PlanarHistoryResource, PlanarHistoryGrant, Body);
relation!(PlanarHistoryParent, PlanarHistoryGrant, PlanarHistoryGrant);
relation!(PlanarHistoryGrantor, Principal, PlanarHistoryGrant);
relation!(PlanarHistoryGrantee, Principal, PlanarHistoryGrant);

pub(super) fn declare<S: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    schema
        .entity(PlanarHistoryGrant::reference())
        .aspect(
            PlanarHistoryGrant::reference(),
            PlanarHistoryGrantFacts::reference(),
        )
        .unique_field(
            PlanarHistoryGrant::reference(),
            PlanarHistoryGrantIdentity::reference(),
        )
        .field(
            PlanarHistoryGrant::reference(),
            PlanarHistoryScopeKey::reference(),
        )
        .field(
            PlanarHistoryGrant::reference(),
            PlanarHistoryAction::reference(),
        )
        .field(
            PlanarHistoryGrant::reference(),
            PlanarHistoryPurpose::reference(),
        )
        .field(
            PlanarHistoryGrant::reference(),
            PlanarHistoryStatus::reference(),
        )
        .field(
            PlanarHistoryGrant::reference(),
            PlanarHistoryNotBefore::reference(),
        )
        .field(
            PlanarHistoryGrant::reference(),
            PlanarHistoryNotAfter::reference(),
        )
        .field(
            PlanarHistoryGrant::reference(),
            PlanarHistoryDepth::reference(),
        )
        .relation(
            PlanarHistoryResource::reference(),
            PlanarHistoryGrant::reference(),
            Body::reference(),
        )
        .relation(
            PlanarHistoryParent::reference(),
            PlanarHistoryGrant::reference(),
            PlanarHistoryGrant::reference(),
        )
        .relation(
            PlanarHistoryGrantor::reference(),
            Principal::reference(),
            PlanarHistoryGrant::reference(),
        )
        .relation(
            PlanarHistoryGrantee::reference(),
            Principal::reference(),
            PlanarHistoryGrant::reference(),
        )
}
