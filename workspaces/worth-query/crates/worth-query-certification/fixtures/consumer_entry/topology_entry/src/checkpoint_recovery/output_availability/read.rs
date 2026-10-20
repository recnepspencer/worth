//! Ordinary native query of the published checked collection record.
use super::*;
use members::{Quantity, QuantityFacts, QuantityKey, QuantitySnapshot};
#[derive(Clone, Debug)]
pub(super) struct Read(pub String);
pub(super) struct Parameters;
worth_query_structured_value_binding!(pub(super) ReadInput for Read {identity:"availability-read-input"});
worth_query_structured_value_binding!(pub(super) ParameterBinding for Parameters {identity:"availability-read-parameters"});
pub(super) struct Query;
pub(super) struct SnapshotSlot;
worth_query_portable_type!(SnapshotSlot => "availability-snapshot-slot");
impl ApplicationQueryMarkerIdentity<Schema> for Query {
    type ParameterBinding = ParameterBinding;
    type ResultBinding = value::SnapshotResultBinding;
    type Scope = Quantity;
    const IDENTIFIER: &'static str = "AvailabilityQuantity";
    const QUERY_TYPE_NAME: &'static str = "availability-quantity-query";
    const SCOPE_TYPE_NAME: &'static str = "Quantity";
}
fn snapshot() -> ApplicationQueryResultFieldRef<
    Query,
    SnapshotSlot,
    Schema,
    Quantity,
    QuantityFacts,
    QuantitySnapshot,
    Snapshot,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("snapshot", QuantitySnapshot::reference())
}
pub(super) fn definition(
) -> ApplicationQueryDefinition<Schema, Query, Parameters, Snapshot, Quantity> {
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        Quantity,
        Snapshot,
        value::SnapshotResultBinding,
    >::new(Quantity::reference())
    .field(snapshot())
    .build();
    ApplicationQueryDefinitionBuilder::declare(ApplicationQueryReference::<
        Schema,
        Query,
        Parameters,
        Snapshot,
        Quantity,
    >::from_declaration())
    .root(Quantity::reference())
    .scope(Quantity::reference())
    .result_shape(shape)
    .cardinality(ApplicationQueryCardinality::ExactlyOne)
    .disclosure(ApplicationQueryDisclosureContract::public())
    .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
    .lanes(ApplicationQueryLaneEligibility::one_shot())
    .public()
    .build()
    .unwrap()
}
impl worth_query_host::facade::primary_graph::WorthQueryApplicationProjection<Schema, Query>
    for Snapshot
{
    fn project(
        row: &worth_query_host::facade::primary_graph::WorthQueryApplicationProjectionRow<
            '_,
            Schema,
            Query,
        >,
    ) -> Result<Self, worth_query_host::facade::primary_graph::WorthQueryApplicationProjectionDenial>
    {
        row.field(snapshot())
    }
}
pub(super) struct Binding;
type Scope = ApplicationQueryFieldScope<
    Schema,
    Quantity,
    QuantityFacts,
    QuantityKey,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationQueryBinding<Schema> for Binding {
    type Input = Read;
    type InputBinding = ReadInput;
    type Query = Query;
    type ParameterBinding = ParameterBinding;
    type ResultBinding = value::SnapshotResultBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "availability-quantity-read";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::bounded(1, 1024);
    fn scope_field() -> ApplicationFieldRef<
        Schema,
        Quantity,
        QuantityFacts,
        QuantityKey,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        QuantityKey::reference()
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
impl ApplicationQueryIntent<Schema> for Read {
    type Binding = Binding;
    fn parameters(&self) -> ApplicationQueryParameterSet<Query> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> Scope {
        Scope::new(QuantityKey::reference(), self.0)
    }
}
