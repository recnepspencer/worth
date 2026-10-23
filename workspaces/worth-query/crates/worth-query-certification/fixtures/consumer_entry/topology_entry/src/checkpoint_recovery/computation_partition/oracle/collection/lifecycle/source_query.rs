//! Fresh authored rows reconstruct optional payload without generated readback.
use super::*;
use facts::{
    BodyEntrySet, EntryFault, EntryNumber, EntryRegion, EntrySet, EntrySetFacts, EntrySetKey,
    EntrySetMember, EntrySetWeight, EntryValueBits, EntryWork, SetEntry, SetEntryFacts,
};
use worth_query_decl::facade::{
    application_query::*, application_schema::*, worth_query_portable_type,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

#[derive(Clone, Debug)]
pub(super) struct SourceRead {
    pub(super) body_key: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SourceRow {
    pub(super) sets: Vec<SourceSet>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SourceSet {
    pub(super) key: String,
    pub(super) weight: u64,
    pub(super) entries: Vec<SourceEntry>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SourceEntry {
    pub(super) number: u64,
    pub(super) region: u64,
    pub(super) value: u64,
    pub(super) work: u64,
    pub(super) fault: u64,
}
pub(super) struct Parameters;
worth_query_structured_value_binding!(pub(super) Input for SourceRead { identity: "collection-source-read.input.v1" });
worth_query_structured_value_binding!(pub(super) ParameterBinding for Parameters { identity: "collection-source-read.parameters.v1" });
worth_query_structured_value_binding!(pub(super) ResultBinding for SourceRow { identity: "collection-source-read.result.v1" });
pub(super) struct Query;
impl<Schema: TopologySchemaBinding> ApplicationQueryMarkerIdentity<Schema> for Query {
    type ParameterBinding = ParameterBinding;
    type ResultBinding = ResultBinding;
    type Scope = Body;
    const IDENTIFIER: &'static str = "CollectionSourceRead";
    const QUERY_TYPE_NAME: &'static str = "collection-source-read.query.v1";
    const SCOPE_TYPE_NAME: &'static str = "Body";
}
pub(super) struct Binding<Schema>(PhantomData<fn() -> Schema>);
impl<Schema: TopologySchemaBinding> ApplicationQueryBinding<Schema> for Binding<Schema> {
    type Input = SourceRead;
    type InputBinding = Input;
    type Query = Query;
    type ParameterBinding = ParameterBinding;
    type ResultBinding = ResultBinding;
    type ScopeBinding = PlanarReadScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "collection-source-read.binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ApplicationFieldRef<
        Schema,
        Body,
        PlanarPosition,
        BodyKey,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        BodyKey::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        ConsumerPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ConsumerPrincipalBinding::reference()
    }
}
impl<Schema: TopologySchemaBinding> ApplicationQueryIntent<Schema> for SourceRead {
    type Binding = Binding<Schema>;
    fn parameters(&self) -> ApplicationQueryParameterSet<Query> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> PlanarReadScope<Schema> {
        PlanarReadScope::new(BodyKey::reference(), self.body_key)
    }
}

pub(super) struct SetsSlot;
pub(super) struct EntriesSlot;
worth_query_portable_type!(SetsSlot => "collection-source-read.sets");
worth_query_portable_type!(EntriesSlot => "collection-source-read.entries");
fn sets<Schema: TopologySchemaBinding>() -> ApplicationQueryResultRelationRef<
    Query,
    SetsSlot,
    Schema,
    BodyEntrySet,
    Body,
    EntrySet,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("sets", BodyEntrySet::reference())
}
fn entries<Schema: TopologySchemaBinding>() -> ApplicationQueryResultRelationRef<
    Query,
    EntriesSlot,
    Schema,
    EntrySetMember,
    EntrySet,
    SetEntry,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("entries", EntrySetMember::reference())
}
pub(super) struct EntrySetKeySlot;
worth_query_portable_type!(EntrySetKeySlot => "collection-source-read.set_key");
fn set_key<Schema: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    Query,
    EntrySetKeySlot,
    Schema,
    EntrySet,
    EntrySetFacts,
    EntrySetKey,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("set_key", EntrySetKey::reference())
}
pub(super) struct EntrySetWeightSlot;
worth_query_portable_type!(EntrySetWeightSlot => "collection-source-read.weight");
fn weight<Schema: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    Query,
    EntrySetWeightSlot,
    Schema,
    EntrySet,
    EntrySetFacts,
    EntrySetWeight,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("weight", EntrySetWeight::reference())
}
pub(super) struct EntryNumberSlot;
worth_query_portable_type!(EntryNumberSlot => "collection-source-read.number");
fn number<Schema: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    Query,
    EntryNumberSlot,
    Schema,
    SetEntry,
    SetEntryFacts,
    EntryNumber,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("number", EntryNumber::reference())
}
pub(super) struct EntryRegionSlot;
worth_query_portable_type!(EntryRegionSlot => "collection-source-read.region");
fn region<Schema: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    Query,
    EntryRegionSlot,
    Schema,
    SetEntry,
    SetEntryFacts,
    EntryRegion,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("region", EntryRegion::reference())
}
pub(super) struct EntryValueBitsSlot;
worth_query_portable_type!(EntryValueBitsSlot => "collection-source-read.value");
fn value<Schema: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    Query,
    EntryValueBitsSlot,
    Schema,
    SetEntry,
    SetEntryFacts,
    EntryValueBits,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("value", EntryValueBits::reference())
}
pub(super) struct EntryWorkSlot;
worth_query_portable_type!(EntryWorkSlot => "collection-source-read.work");
fn work<Schema: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    Query,
    EntryWorkSlot,
    Schema,
    SetEntry,
    SetEntryFacts,
    EntryWork,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("work", EntryWork::reference())
}
pub(super) struct EntryFaultSlot;
worth_query_portable_type!(EntryFaultSlot => "collection-source-read.fault");
fn fault<Schema: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    Query,
    EntryFaultSlot,
    Schema,
    SetEntry,
    SetEntryFacts,
    EntryFault,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("fault", EntryFault::reference())
}

pub(super) fn declare<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let entry = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        SetEntry,
        SourceRow,
        ResultBinding,
    >::new(SetEntry::reference())
    .field(number())
    .field(region())
    .field(value())
    .field(work())
    .field(fault());
    let set = ApplicationQueryResultShapeBuilder::<Schema, Query, EntrySet, SourceRow, ResultBinding>::new(EntrySet::reference()).field(set_key()).field(weight()).relation(entries(), entry);
    let shape =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Body, SourceRow, ResultBinding>::new(
            Body::reference(),
        )
        .relation(sets(), set)
        .build();
    let definition = ApplicationQueryDefinitionBuilder::declare(ApplicationQueryReference::<
        Schema,
        Query,
        Parameters,
        SourceRow,
        Body,
    >::from_declaration())
    .root(Body::reference())
    .scope(Body::reference())
    .result_shape(shape)
    .cardinality(ApplicationQueryCardinality::ExactlyOne)
    .disclosure(ApplicationQueryDisclosureContract::public())
    .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
    .lanes(ApplicationQueryLaneEligibility::one_shot())
    .public()
    .build()
    .unwrap();
    schema
        .application_query(definition)
        .application_query_binding::<Binding<Schema>>()
}
impl<Schema: TopologySchemaBinding> WorthQueryApplicationProjection<Schema, Query> for SourceRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let sets = row
            .many(sets())?
            .iter()
            .map(|set| {
                let entries = set
                    .many(entries())?
                    .iter()
                    .map(|entry| {
                        Ok(SourceEntry {
                            number: entry.field(number())?,
                            region: entry.field(region())?,
                            value: entry.field(value())?,
                            work: entry.field(work())?,
                            fault: entry.field(fault())?,
                        })
                    })
                    .collect::<Result<Vec<_>, WorthQueryApplicationProjectionDenial>>()?;
                Ok(SourceSet {
                    key: set.field(set_key())?,
                    weight: set.field(weight())?,
                    entries,
                })
            })
            .collect::<Result<Vec<_>, WorthQueryApplicationProjectionDenial>>()?;
        Ok(Self { sets })
    }
}
impl SourceRow {
    /// The same canonical region/entry order and even-region weight as the domain owner.
    pub(super) fn total(&self, set_key: &str) -> u64 {
        let mut selected = self.sets.iter().filter(|set| set.key == set_key);
        let set = selected
            .next()
            .expect("the fresh source holds its selected owned set");
        assert!(selected.next().is_none());
        assert!(
            !set.entries.is_empty(),
            "this fixture's corrected set is nonempty"
        );
        let mut entries = set.entries.clone();
        entries.sort_by_key(|entry| (entry.region, entry.number));
        let mut regions = std::collections::BTreeMap::<u64, f64>::new();
        for entry in entries {
            assert_eq!(
                entry.fault, 0,
                "faulted authored source cannot reconstruct a value"
            );
            let sum = regions.entry(entry.region).or_insert_with(|| {
                if entry.region.is_multiple_of(2) {
                    f64::from_bits(set.weight)
                } else {
                    -0.0
                }
            });
            *sum += f64::from_bits(entry.value);
        }
        let value = regions.values().fold(-0.0, |sum, value| sum + value);
        assert!(
            value.is_finite()
                && value.fract() == 0.0
                && (1.0..=(1u64 << 53) as f64).contains(&value)
        );
        value as u64
    }
}
