// Both fixture compositions reuse these rows and their single binding set.
use worth_query_decl::facade::application_schema::{
    fragment::{
        self, entity, field, relation, Aspect, BindingSet, Entity, FieldSelection, Fragment,
        Relation, Scalar, Shape,
    },
    ApplicationEntityRef, ApplicationRelationRef, StringApplicationValueBinding,
    U64ApplicationValueBinding, ValueDecodeAdmission,
};
pub struct FactsRecords;
impl Fragment for FactsRecords {
    const NAME: &'static str = "entry_facts";
    type Bindings = FactsBindings;
    type Members = (EntrySet, SetEntry);
}
pub struct FactsBindings;
impl BindingSet for FactsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<String> for FactsBindings {
    type Binding = StringApplicationValueBinding;
    const WRITABLE: bool = false;
    const EQUALITY_QUERYABLE: bool = true;
}
impl Scalar<u64> for FactsBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
}
#[derive(Clone, Copy, Debug)]
pub struct EntrySet;
impl Entity for EntrySet {
    const NAME: &'static str = "EntrySet";
    type Aspects = (EntrySetFacts,);
    type Relations = (EntrySetMember,);
}
impl EntrySet {
    pub fn reference<Schema: TopologyModelBinding>() -> ApplicationEntityRef<Schema, Self> {
        entity(FactsRecords, Self)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct SetEntry;
impl Entity for SetEntry {
    const NAME: &'static str = "SetEntry";
    type Aspects = (SetEntryFacts,);
    type Relations = ();
}
impl SetEntry {
    pub fn reference<Schema: TopologyModelBinding>() -> ApplicationEntityRef<Schema, Self> {
        entity(FactsRecords, Self)
    }
}
pub struct EntrySetFacts;
impl Aspect for EntrySetFacts {
    const NAME: &'static str = "EntrySetFacts";
    type Row = EntrySetValue;
}
pub struct SetEntryFacts;
impl Aspect for SetEntryFacts {
    const NAME: &'static str = "SetEntryFacts";
    type Row = SetEntryValue;
}
pub struct EntrySetValue {
    pub key: String,
    pub weight: u64,
}
pub struct SetEntryValue {
    pub work: u64,
    pub number: u64,
    pub region: u64,
    pub value_bits: u64,
    pub fault: u64,
}
impl fragment::Record<EntrySetValue> for FactsBindings {
    fn record() -> impl Shape<EntrySetValue, Self> {
        field("EntrySetKey", |row: &EntrySetValue| row.key.clone())
            .and("EntrySetWeight", |row: &EntrySetValue| row.weight)
            .build(|key, weight, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(EntrySetValue { key, weight })
            })
    }
}
impl fragment::Record<SetEntryValue> for FactsBindings {
    fn record() -> impl Shape<SetEntryValue, Self> {
        field("EntryWork", |row: &SetEntryValue| row.work)
            .read_only()
            .and("EntryNumber", |row: &SetEntryValue| row.number)
            .and("EntryRegion", |row: &SetEntryValue| row.region)
            .and("EntryValueBits", |row: &SetEntryValue| row.value_bits)
            .and("EntryFault", |row: &SetEntryValue| row.fault)
            .build(
                |work, number, region, value_bits, fault, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(SetEntryValue {
                        work,
                        number,
                        region,
                        value_bits,
                        fault,
                    })
                },
            )
    }
}
pub type EntrySetKey = FieldSelection<FactsRecords, EntrySet, EntrySetFacts, String>;
pub type EntrySetWeight = FieldSelection<FactsRecords, EntrySet, EntrySetFacts, u64>;
pub type EntryWork = FieldSelection<FactsRecords, SetEntry, SetEntryFacts, u64>;
pub type EntryNumber = FieldSelection<FactsRecords, SetEntry, SetEntryFacts, u64>;
pub type EntryRegion = FieldSelection<FactsRecords, SetEntry, SetEntryFacts, u64>;
pub type EntryValueBits = FieldSelection<FactsRecords, SetEntry, SetEntryFacts, u64>;
pub type EntryFault = FieldSelection<FactsRecords, SetEntry, SetEntryFacts, u64>;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntrySetMember;
impl Relation for EntrySetMember {
    const NAME: &'static str = "EntrySetMember";
    type To = SetEntry;
}
impl EntrySetMember {
    pub fn reference<Schema: TopologyModelBinding>(
    ) -> ApplicationRelationRef<Schema, Self, EntrySet, SetEntry> {
        relation(FactsRecords, EntrySet, Self)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BodyEntrySet;
impl Relation for BodyEntrySet {
    const NAME: &'static str = "BodyEntrySet";
    type To = EntrySet;
}
impl BodyEntrySet {
    pub fn reference<Schema: TopologyModelBinding>(
    ) -> ApplicationRelationRef<Schema, Self, Body, EntrySet> {
        ApplicationRelationRef::from_schema_identifiers(
            Self::NAME,
            <Body as Entity>::NAME,
            <EntrySet as Entity>::NAME,
            fragment::relation_integrity::<Self>(),
        )
    }
}
