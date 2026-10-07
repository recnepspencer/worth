//! The facts the region totals are computed from: named sets of entries in
//! the primary graph, how a test seeds them and how an owner reads them
//! through the reader it is lent.

use serde::Serialize;
use worth_query_decl::facade::application_program::ApplicationComputationPartition;
use worth_query_decl::facade::application_schema::{
    ApplicationRelationIntegrity, ApplicationRelationRef, ApplicationSchemaDeclarationBuilder,
    OperationReads, StringApplicationValueBinding, U64ApplicationValueBinding,
};
use worth_query_decl::facade::{worth_query_aspect, worth_query_entity, worth_query_field};
use worth_query_host::facade::application_contribution::{
    ChargedBytes, PartitionItemId, WorthQueryComputationInputDenial,
    WorthQueryComputationPartitionPlan, WorthQueryComputationReadDenial,
    WorthQueryComputationReader,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryInvariantEntityIdentity,
    WorthQueryPrimaryGraphBootstrap,
};

use super::demand::TotalRegions;
use super::probe::ProbeFault;
use super::*;

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

/// One entry of a set, as a test writes it.
#[derive(Clone, Copy, Debug)]
pub(super) struct RegionEntry {
    /// The entry's own identity, whatever its place in the set.
    pub(super) id: u64,
    pub(super) region: u32,
    pub(super) value: f64,
    /// Work the region's kernel charges before it reads the value.
    pub(super) work: usize,
    pub(super) fault: Option<RegionFault>,
}

/// What a region's kernel does instead of reading an entry.
#[derive(Clone, Copy, Debug)]
pub(super) enum RegionFault {
    /// Refuses the region, naming it.
    Refuse,
    Panic,
    /// The probing owner's faults. The totals owner reads through them.
    Probe(ProbeFault),
}

/// The first code of a held result: the values held follow it.
const HOLD: u64 = 5;

impl RegionFault {
    /// The fact an entry's fault is: zero is no fault.
    pub(super) fn code(fault: Option<Self>) -> u64 {
        match fault {
            None => 0,
            Some(Self::Refuse) => 1,
            Some(Self::Panic) => 2,
            Some(Self::Probe(ProbeFault::NoKey)) => 3,
            Some(Self::Probe(ProbeFault::PoisonReducer)) => 4,
            Some(Self::Probe(ProbeFault::Hold(values))) => HOLD + values as u64,
        }
    }

    fn of_code(code: u64) -> Option<Self> {
        match code {
            0 => None,
            1 => Some(Self::Refuse),
            2 => Some(Self::Panic),
            3 => Some(Self::Probe(ProbeFault::NoKey)),
            4 => Some(Self::Probe(ProbeFault::PoisonReducer)),
            held => Some(Self::Probe(ProbeFault::Hold((held - HOLD) as usize))),
        }
    }
}

/// The sets a test installs, each under the name its demands ask for.
pub(super) type Sets<'sets> = &'sets [(&'sets str, &'sets [RegionEntry])];

/// Seeds every set and its entries. An entry's entity is named by its place,
/// so a set holds its entries in the order the test wrote them, whatever
/// their numbers.
pub(super) fn seed(graph: &mut Graph, sets: Sets<'_>) {
    for (set, entries) in sets {
        seed_set(graph, set, -0.0);
        for (place, entry) in entries.iter().enumerate() {
            let entry_key = format!("entries-{set}-{place}");
            seed_entry(graph, &entry_key, entry);
            seed_member(graph, set, &entry_key);
        }
    }
}

pub(super) type Graph = WorthQueryPrimaryGraphBootstrap<CheckpointSchema>;

/// Seeds the set named `set`, lending `weight`.
pub(super) fn seed_set(graph: &mut Graph, set: &str, weight: f64) {
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                EntrySet::reference::<CheckpointSchema>(),
                key(&format!("entries-{set}")),
            )
            .field(EntrySetKey::reference(), set.to_owned())
            .field(EntrySetWeight::reference(), weight.to_bits()),
        )
        .unwrap();
}

/// Seeds one entry under its entity's own name.
pub(super) fn seed_entry(graph: &mut Graph, entry_key: &str, entry: &RegionEntry) {
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                SetEntry::reference::<CheckpointSchema>(),
                key(entry_key),
            )
            .field(EntryNumber::reference(), entry.id)
            .field(EntryRegion::reference(), u64::from(entry.region))
            .field(EntryValueBits::reference(), entry.value.to_bits())
            .field(EntryWork::reference(), entry.work as u64)
            .field(EntryFault::reference(), RegionFault::code(entry.fault)),
        )
        .unwrap();
}

/// Makes the seeded entry a member of the seeded set.
pub(super) fn seed_member(graph: &mut Graph, set: &str, entry_key: &str) {
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            EntrySetMember::reference::<CheckpointSchema>(),
            format!("{entry_key}-in-{set}"),
            key(&format!("entries-{set}")),
            key(entry_key),
        ))
        .unwrap();
}

pub(super) fn key<Entity>(key: &str) -> WorthQueryApplicationEntityKey<CheckpointSchema, Entity> {
    WorthQueryApplicationEntityKey::new(key).unwrap()
}

/// The reader an owner of the region totals is lent, under the operation
/// whose decision runs it.
pub(super) type Reader<'call, 'reader, 'runtime, Operation = TotalRegions> =
    WorthQueryComputationReader<'call, 'reader, 'runtime, CheckpointSchema, Operation>;
/// The computation's input: the set whose entries it totals.
pub(super) type Set = WorthQueryInvariantEntityIdentity<CheckpointSchema, EntrySet>;
pub(super) type InputDenial = WorthQueryComputationInputDenial<u32>;
type Read<Value> = Result<Value, WorthQueryComputationReadDenial>;

/// One entry as an owner holds it between its calls: its number, and where
/// its other facts are read. It encodes both, so an entry made again under
/// the same number is a changed entry.
#[derive(Serialize)]
pub(super) struct Entry {
    pub(super) number: u64,
    entity: WorthQueryInvariantEntityIdentity<CheckpointSchema, SetEntry>,
}

impl Entry {
    /// Independent installations bind these same Native entities under
    /// different projection authorities; compare the carried entity and kind.
    pub(super) fn same_binding_as(&self, other: &Self) -> bool {
        self.number == other.number
            && self.entity.entity_id() == other.entity.entity_id()
            && self.entity.entity_name() == other.entity.entity_name()
    }
}

impl ApplicationComputationPartition for Entry {
    const IDENTITY: &'static str = "checkpoint-region-entry";
}

/// An entry is held inline; its identity shares its entity's name and owns
/// no allocation.
impl ChargedBytes for Entry {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

// A seeded entry holds every fact, so a missing one is a broken fixture.
macro_rules! fact {
    ($reader:expr, $entity:expr, $field:ident) => {
        $reader
            .field($entity, $field::reference())?
            .expect("a seeded entry holds every fact")
    };
}

/// The set's entries, each named by its number. A member whose entry was
/// deleted holds no number and is no entry of the set.
pub(super) fn entries<Operation>(
    reader: &mut Reader<'_, '_, '_, Operation>,
    set: &Set,
) -> Read<WorthQueryComputationPartitionPlan<Entry>>
where
    EntrySetMember: OperationReads<Operation>,
    EntryNumber: OperationReads<Operation>,
{
    let mut entries = Vec::new();
    for member in reader.relations_from(EntrySetMember::reference(), set)? {
        let entity = member.into_to();
        if let Some(number) = reader.field(&entity, EntryNumber::reference())? {
            entries.push(Entry { number, entity });
        }
    }
    Ok(WorthQueryComputationPartitionPlan::keyed(
        entries,
        |entry| PartitionItemId(entry.number),
    ))
}

/// The float the set lends some of its regions.
#[cfg(feature = "test-query-execution-observer")]
pub(super) fn weight<Operation>(reader: &mut Reader<'_, '_, '_, Operation>, set: &Set) -> Read<f64>
where
    EntrySetWeight: OperationReads<Operation>,
{
    Ok(f64::from_bits(fact!(reader, set, EntrySetWeight)))
}

pub(super) fn region<Operation>(
    reader: &mut Reader<'_, '_, '_, Operation>,
    entry: &Entry,
) -> Read<u32>
where
    EntryRegion: OperationReads<Operation>,
{
    let region = fact!(reader, &entry.entity, EntryRegion);
    Ok(u32::try_from(region).expect("a seeded region is 32 bits"))
}

pub(super) fn fault<Operation>(
    reader: &mut Reader<'_, '_, '_, Operation>,
    entry: &Entry,
) -> Read<Option<RegionFault>>
where
    EntryFault: OperationReads<Operation>,
{
    Ok(RegionFault::of_code(fact!(
        reader,
        &entry.entity,
        EntryFault
    )))
}

/// What a region's kernel is handed of one entry.
pub(super) struct EntryData {
    pub(super) value: f64,
    pub(super) work: usize,
    pub(super) fault: Option<RegionFault>,
}

/// An entry's data is held inline.
impl ChargedBytes for EntryData {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

/// The data of a partition's entries, in the order it holds them.
pub(super) fn gathered<'gather, Operation>(
    reader: &mut Reader<'_, '_, '_, Operation>,
    entries: impl Iterator<Item = (PartitionItemId, &'gather Entry)>,
) -> Read<Vec<EntryData>>
where
    EntryValueBits: OperationReads<Operation>,
    EntryWork: OperationReads<Operation>,
    EntryFault: OperationReads<Operation>,
{
    entries
        .map(|(_, entry)| -> Read<EntryData> {
            Ok(EntryData {
                value: f64::from_bits(fact!(reader, &entry.entity, EntryValueBits)),
                work: fact!(reader, &entry.entity, EntryWork) as usize,
                fault: fault(reader, entry)?,
            })
        })
        .collect()
}

/// The entry's complete incoming membership, used by the wide-key oracle.
#[cfg(feature = "test-query-execution-observer")]
pub(super) fn membership_count<Operation>(
    reader: &mut Reader<'_, '_, '_, Operation>,
    entry: &Entry,
) -> Read<usize>
where
    EntrySetMember: OperationReads<Operation>,
{
    let mut co_memberships = 0;
    for membership in reader.relations_to(EntrySetMember::reference(), &entry.entity)? {
        co_memberships += reader
            .relations_from(EntrySetMember::reference(), &membership.into_from())?
            .len();
    }
    Ok(co_memberships)
}
