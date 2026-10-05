//! The facts the region totals are computed from: named sets of entries in
//! the primary graph, how a test seeds them and how an owner reads them
//! through the reader it is lent.

use worth_query_decl::facade::application_schema::{
    ApplicationRelationIntegrity, ApplicationRelationRef, ApplicationSchemaDeclarationBuilder,
    StringApplicationValueBinding, U64ApplicationValueBinding,
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

// Every fact of an entry is one unsigned value. The number is the entry's own
// identity, the value is a float's bits and the fault is `RegionFault`'s code.
macro_rules! entry_facts {
    ($($field:ident),+) => {$(
        worth_query_field!(
            pub(super) $field for Schema: TopologySchemaBinding, SetEntry, SetEntryFacts:
            u64 => U64ApplicationValueBinding, read_only, equality
        );
    )+};
}
entry_facts!(
    EntryNumber,
    EntryRegion,
    EntryValueBits,
    EntryWork,
    EntryFault
);

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
    fn code(fault: Option<Self>) -> u64 {
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
pub(super) fn seed(graph: &mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>, sets: Sets<'_>) {
    for (set, entries) in sets {
        let set_key = format!("entries-{set}");
        graph
            .bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    EntrySet::reference::<CheckpointSchema>(),
                    key(&set_key),
                )
                .field(EntrySetKey::reference(), (*set).to_owned()),
            )
            .unwrap();
        for (place, entry) in entries.iter().enumerate() {
            let entry_key = format!("{set_key}-{place}");
            graph
                .bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        SetEntry::reference::<CheckpointSchema>(),
                        key(&entry_key),
                    )
                    .field(EntryNumber::reference(), entry.id)
                    .field(EntryRegion::reference(), u64::from(entry.region))
                    .field(EntryValueBits::reference(), entry.value.to_bits())
                    .field(EntryWork::reference(), entry.work as u64)
                    .field(EntryFault::reference(), RegionFault::code(entry.fault)),
                )
                .unwrap();
            graph
                .bind_relation(WorthQueryApplicationRelationSeed::new(
                    EntrySetMember::reference::<CheckpointSchema>(),
                    format!("{entry_key}-member"),
                    key(&set_key),
                    key(&entry_key),
                ))
                .unwrap();
        }
    }
}

fn key<Entity>(key: &str) -> WorthQueryApplicationEntityKey<CheckpointSchema, Entity> {
    WorthQueryApplicationEntityKey::new(key).unwrap()
}

/// The reader an owner of the region totals is lent.
pub(super) type Reader<'call, 'reader, 'runtime> =
    WorthQueryComputationReader<'call, 'reader, 'runtime, CheckpointSchema, TotalRegions>;
/// The computation's input: the set whose entries it totals.
pub(super) type Set = WorthQueryInvariantEntityIdentity<CheckpointSchema, EntrySet>;
pub(super) type InputDenial = WorthQueryComputationInputDenial<u32>;
type Read<Value> = Result<Value, WorthQueryComputationReadDenial>;

/// One entry as an owner holds it between its calls: its number, and where
/// its other facts are read.
pub(super) struct Entry {
    pub(super) number: u64,
    entity: WorthQueryInvariantEntityIdentity<CheckpointSchema, SetEntry>,
}

// A seeded entry holds every fact, so a missing one is a broken fixture.
macro_rules! fact {
    ($reader:expr, $entity:expr, $field:ident) => {
        $reader
            .field($entity, $field::reference())?
            .expect("a seeded entry holds every fact")
    };
}

/// The set's entries, each named by its number.
pub(super) fn entries(
    reader: &mut Reader<'_, '_, '_>,
    set: &Set,
) -> Read<WorthQueryComputationPartitionPlan<Entry>> {
    let mut entries = Vec::new();
    for member in reader.relations_from(EntrySetMember::reference(), set)? {
        let entity = member.into_to();
        let number = fact!(reader, &entity, EntryNumber);
        entries.push(Entry { number, entity });
    }
    Ok(WorthQueryComputationPartitionPlan::keyed(
        entries,
        |entry| PartitionItemId(entry.number),
    ))
}

pub(super) fn region(reader: &mut Reader<'_, '_, '_>, entry: &Entry) -> Read<u32> {
    let region = fact!(reader, &entry.entity, EntryRegion);
    Ok(u32::try_from(region).expect("a seeded region is 32 bits"))
}

pub(super) fn fault(reader: &mut Reader<'_, '_, '_>, entry: &Entry) -> Read<Option<RegionFault>> {
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
pub(super) fn gathered<'gather>(
    reader: &mut Reader<'_, '_, '_>,
    entries: impl Iterator<Item = (PartitionItemId, &'gather Entry)>,
) -> Read<Vec<EntryData>> {
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
