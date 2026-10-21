//! Governed business-ID readback, independent of the retirement implementation.
use super::{
    super::{
        query::{DocumentQueryParameters, DocumentQueryParametersBinding},
        schema::*,
    },
    declaration::*,
};
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::{
        WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
        WorthQueryApplicationProjectionRow,
    },
    worth_query_application_query, worth_query_portable_type, worth_query_structured_value_binding,
};
#[derive(Clone)]
pub struct SnapshotRead(pub u64);
#[derive(Clone, Debug)]
pub struct SnapshotRow(pub u64);
worth_query_portable_type!(SnapshotRow => "worth.query.body-court.snapshot-row.v1");
worth_query_structured_value_binding!(pub SnapshotReadBinding for SnapshotRead { identity: "worth.query.body-court.snapshot-read.v1" });
worth_query_structured_value_binding!(pub SnapshotRowBinding for SnapshotRow { identity: "worth.query.body-court.snapshot-row.v1" });
pub struct IdentitySlot;
worth_query_portable_type!(IdentitySlot => "worth.query.body-court.snapshot-identity-slot.v1");
worth_query_application_query!(pub SnapshotQuery for Schema: BodySchemaBinding,
    identity "SnapshotQuery", parameters DocumentQueryParametersBinding,
    result SnapshotRowBinding, scope Snapshot => "Snapshot", name "body_snapshot_read");
fn identity<S: BodySchemaBinding>() -> ApplicationQueryResultFieldRef<
    SnapshotQuery,
    IdentitySlot,
    S,
    Snapshot,
    SnapshotFacts,
    SnapshotIdentity,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("identity", SnapshotIdentity::reference())
}
pub fn definition<const M: u64>() -> ApplicationQueryDefinition<
    DocumentRetentionSchema<M>,
    SnapshotQuery,
    DocumentQueryParameters,
    SnapshotRow,
    Snapshot,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        DocumentRetentionSchema<M>,
        SnapshotQuery,
        Snapshot,
        SnapshotRow,
        SnapshotRowBinding,
    >::new(Snapshot::reference())
    .field(identity())
    .build();
    ApplicationQueryDefinitionBuilder::declare(SnapshotQuery::reference())
        .root(Snapshot::reference())
        .scope(Snapshot::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}
impl<const M: u64> WorthQueryApplicationProjection<DocumentRetentionSchema<M>, SnapshotQuery>
    for SnapshotRow
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, DocumentRetentionSchema<M>, SnapshotQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self(row.field(identity())?))
    }
}
type Scope<const M: u64> = ApplicationQueryFieldScope<
    DocumentRetentionSchema<M>,
    Snapshot,
    SnapshotFacts,
    SnapshotIdentity,
    u64,
    ReadOnly,
    NoApplicationUnit,
>;
pub struct SnapshotQueryBinding<const M: u64>;
impl<const M: u64> ApplicationQueryBinding<DocumentRetentionSchema<M>> for SnapshotQueryBinding<M> {
    type Input = SnapshotRead;
    type InputBinding = SnapshotReadBinding;
    type Query = SnapshotQuery;
    type ParameterBinding = DocumentQueryParametersBinding;
    type ResultBinding = SnapshotRowBinding;
    type ScopeBinding = Scope<M>;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.body-court.snapshot-query-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ApplicationFieldRef<
        DocumentRetentionSchema<M>,
        Snapshot,
        SnapshotFacts,
        SnapshotIdentity,
        u64,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        SnapshotIdentity::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DocumentRetentionSchema<M>,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}
impl<const M: u64> ApplicationQueryIntent<DocumentRetentionSchema<M>> for SnapshotRead {
    type Binding = SnapshotQueryBinding<M>;
    fn parameters(&self) -> ApplicationQueryParameterSet<SnapshotQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> Scope<M> {
        Scope::new(SnapshotIdentity::reference(), self.0)
    }
}
