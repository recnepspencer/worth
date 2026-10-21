// The same declared entry vocabulary is compiled by both fixture compositions.
use worth_query_decl::facade::application_schema::{
    ApplicationRelationIntegrity, ApplicationRelationRef, ApplicationSchemaDeclarationBuilder,
    StringApplicationValueBinding, U64ApplicationValueBinding,
};
use worth_query_decl::facade::{worth_query_aspect, worth_query_entity, worth_query_field};
worth_query_entity!(pub(super) EntrySet for Schema: TopologySchemaBinding);
worth_query_entity!(pub(super) SetEntry for Schema: TopologySchemaBinding);
worth_query_aspect!(
    pub(super) EntrySetFacts for Schema: TopologySchemaBinding, EntrySet;
    identity = AspectIdentity(0x9174_1031),
    revision = AspectContractRevision(1),
);
worth_query_aspect!(
    pub(super) SetEntryFacts for Schema: TopologySchemaBinding, SetEntry;
    identity = AspectIdentity(0x9174_1032),
    revision = AspectContractRevision(1),
);
worth_query_field!(
    pub(super) EntrySetKey for Schema: TopologySchemaBinding, EntrySet, EntrySetFacts:
    String => StringApplicationValueBinding, read_only, equality
);
// A float's bits a set lends some of its regions.
worth_query_field!(
    pub(super) EntrySetWeight for Schema: TopologySchemaBinding, EntrySet, EntrySetFacts:
    u64 => U64ApplicationValueBinding, read_write, equality
);

// Every fact of an entry is one unsigned value. The number is the entry's own
// identity, the value is a float's bits and the fault is `RegionFault`'s code.
// The number, region, value and fault are edited after seeding.
macro_rules! entry_facts {
    ($posture:ident: $($field:ident),+) => {$(
        worth_query_field!(
            pub(super) $field for Schema: TopologySchemaBinding, SetEntry, SetEntryFacts:
            u64 => U64ApplicationValueBinding, $posture, equality
        );
    )+};
}
entry_facts!(read_only: EntryWork);
entry_facts!(read_write: EntryNumber, EntryRegion, EntryValueBits, EntryFault);

/// A set holds its entries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct EntrySetMember;
impl EntrySetMember {
    pub(super) fn reference<Schema: TopologySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, EntrySet, SetEntry> {
        ApplicationRelationRef::from_schema_identifiers(
            "EntrySetMember",
            "EntrySet",
            "SetEntry",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}

pub(super) fn declare<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .entity(EntrySet::reference::<Schema>())
        .aspect(EntrySet::reference(), EntrySetFacts::reference())
        .field(EntrySet::reference(), EntrySetKey::reference())
        .field(EntrySet::reference(), EntrySetWeight::reference())
        .entity(SetEntry::reference::<Schema>())
        .aspect(SetEntry::reference(), SetEntryFacts::reference())
        .field(SetEntry::reference(), EntryNumber::reference())
        .field(SetEntry::reference(), EntryRegion::reference())
        .field(SetEntry::reference(), EntryValueBits::reference())
        .field(SetEntry::reference(), EntryWork::reference())
        .field(SetEntry::reference(), EntryFault::reference())
        .relation(
            EntrySetMember::reference(),
            EntrySet::reference(),
            SetEntry::reference(),
        )
}
