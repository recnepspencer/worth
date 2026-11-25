//! Schema and source query for Directory Home indexed-selection reopen court.
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
use worth_query_host::facade::{
    declaration::{application_query::*, authentication::*},
    primary_graph::*,
    *,
};

#[derive(Clone, Copy)]
pub struct IndexedSelectionSchema;
impl Fragment for IndexedSelectionSchema {
    const NAME: &'static str = "IndexedSelectionReopenSchema";
    type Bindings = NoBindings;
    type Members = (IndexedSelectionContribution,);
}
impl ApplicationSchema for IndexedSelectionSchema {
    const OWNER: &'static str = "worth.query.test.indexed-selection-reopen";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
    fn visit_correction<
        V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        fragment::visit_correction(Self, source, visitor)
    }
}
impl ApplicationSchemaComposition for IndexedSelectionSchema {
    type Contributions = <Self as Fragment>::Members;
}

pub struct IndexedSelectionContribution;
impl ApplicationSchemaContribution<IndexedSelectionSchema> for IndexedSelectionContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("indexed-selection-reopen.contribution.v1");
    type Members = (Records, super::program::Contracts);
}

#[derive(Clone, Copy)]
pub struct Records;
impl Fragment for Records {
    const NAME: &'static str = "indexed_selection_records";
    type Bindings = RecordBindings;
    type Members = (ExternalMapping, Principal, Account, Derived);
}
pub struct RecordBindings;
impl BindingSet for RecordBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for RecordBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for RecordBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const WRITABLE: bool = true;
}
impl Scalar<u64> for RecordBindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = false;
}
impl Scalar<String> for RecordBindings {
    type Binding = StringApplicationValueBinding;
    const WRITABLE: bool = true;
}

#[derive(Clone, Copy)]
pub struct ExternalMapping;
impl Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
impl ExternalMapping {
    pub fn reference() -> ApplicationEntityRef<IndexedSelectionSchema, Self> {
        entity(Records, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = ();
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<IndexedSelectionSchema, Self> {
        entity(Records, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Account;
impl Entity for Account {
    const NAME: &'static str = "Account";
    type Aspects = (AccountFacts,);
    type Relations = ();
}
impl Account {
    pub fn reference() -> ApplicationEntityRef<IndexedSelectionSchema, Self> {
        entity(Records, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Derived;
impl Entity for Derived {
    const NAME: &'static str = "Derived";
    type Aspects = (DerivedFacts,);
    type Relations = ();
}
impl Derived {
    pub fn reference() -> ApplicationEntityRef<IndexedSelectionSchema, Self> {
        entity(Records, Self)
    }
}

pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityRow;
}
pub struct ExternalIdentityRow {
    pub external_id: WorthQueryExternalPrincipalIdentity,
    pub status: WorthQueryPrincipalMappingStatus,
}
impl Record<ExternalIdentityRow> for RecordBindings {
    fn record() -> impl Shape<ExternalIdentityRow, Self> {
        field("ExternalIdentityField", |r: &ExternalIdentityRow| {
            r.external_id.clone()
        })
        .equality_queryable(true)
        .and("MappingStatusField", |r: &ExternalIdentityRow| r.status)
        .build(|external_id, status, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(ExternalIdentityRow {
                external_id,
                status,
            })
        })
    }
}
pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference(
    ) -> ApplicationRelationRef<IndexedSelectionSchema, Self, ExternalMapping, Principal> {
        relation(Records, ExternalMapping, Self)
    }
}
pub struct PrincipalFacts;
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalRow;
}
pub struct PrincipalRow {
    pub id: u64,
}
impl Record<PrincipalRow> for RecordBindings {
    fn record() -> impl Shape<PrincipalRow, Self> {
        field("PrincipalIdentityField", |r: &PrincipalRow| r.id)
            .equality_queryable(true)
            .build(|id, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalRow { id })
            })
    }
}
pub struct AccountFacts;
impl Aspect for AccountFacts {
    const NAME: &'static str = "AccountFacts";
    type Row = AccountRow;
}
pub struct AccountRow {
    pub key: String,
    pub status: String,
}
impl Record<AccountRow> for RecordBindings {
    fn record() -> impl Shape<AccountRow, Self> {
        field("AccountKey", |r: &AccountRow| r.key.clone())
            .equality_queryable(true)
            .and("AccountStatus", |r: &AccountRow| r.status.clone())
            .equality_queryable(true)
            .build(|key, status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(AccountRow { key, status })
            })
    }
}
pub struct DerivedFacts;
impl Aspect for DerivedFacts {
    const NAME: &'static str = "DerivedFacts";
    type Row = DerivedRow;
}
pub struct DerivedRow {
    pub key: String,
    pub marker: String,
}
impl Record<DerivedRow> for RecordBindings {
    fn record() -> impl Shape<DerivedRow, Self> {
        field("DerivedKey", |r: &DerivedRow| r.key.clone())
            .equality_queryable(true)
            .and("DerivedMarker", |r: &DerivedRow| r.marker.clone())
            .build(|key, marker, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(DerivedRow { key, marker })
            })
    }
}

pub type ExternalIdentityField =
    FieldSelection<Records, ExternalMapping, ExternalIdentity, WorthQueryExternalPrincipalIdentity>;
pub type MappingStatusField =
    FieldSelection<Records, ExternalMapping, ExternalIdentity, WorthQueryPrincipalMappingStatus>;
pub type PrincipalIdentityField = FieldSelection<Records, Principal, PrincipalFacts, u64>;
pub type AccountKey = FieldSelection<Records, Account, AccountFacts, String>;
pub type AccountStatus = FieldSelection<Records, Account, AccountFacts, String>;
pub type DerivedKey = FieldSelection<Records, Derived, DerivedFacts, String>;
pub type DerivedMarker = FieldSelection<Records, Derived, DerivedFacts, String>;

pub struct OperatorPrincipalBinding;
impl OperatorPrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        IndexedSelectionSchema,
        Self,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "OperatorPrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatusField::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentityField::at("PrincipalIdentityField"),
                ),
            },
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyRow(pub String);
pub struct QueryParams;
pub struct KeySlot;
worth_query_portable_type!(KeySlot => "indexed-selection-reopen.key-slot.v1");
worth_query_structured_value_binding!(pub QueryParamsBinding for QueryParams { identity: "indexed-selection-reopen.query-params.v1" });
worth_query_structured_value_binding!(pub KeyRowBinding for KeyRow { identity: "indexed-selection-reopen.key-row.v1" });
worth_query_application_query!(pub AccountQuery for IndexedSelectionSchema, identity "indexed-selection-reopen.account-query.v1",
    parameters QueryParamsBinding, result KeyRowBinding, scope Account => "Account", name "account_key");
pub fn key_field(
) -> ApplicationQueryResultFieldRef<AccountQuery, KeySlot, IndexedSelectionSchema, AccountKey> {
    ApplicationQueryResultFieldRef::new("key", AccountKey::at("AccountKey"))
}
pub fn account_query_definition() -> ApplicationQueryDefinitionBuilder<
    IndexedSelectionSchema,
    AccountQuery,
    QueryParams,
    KeyRow,
    Account,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        IndexedSelectionSchema,
        AccountQuery,
        Account,
        KeyRow,
        KeyRowBinding,
    >::new(Account::reference())
    .field(key_field())
    .build();
    ApplicationQueryDefinitionBuilder::declare(AccountQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
}
impl WorthQueryApplicationProjection<IndexedSelectionSchema, AccountQuery> for KeyRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, IndexedSelectionSchema, AccountQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        row.field(key_field()).map(KeyRow)
    }
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadAccount {
    pub key: String,
}
worth_query_structured_value_binding!(pub ReadAccountBinding for ReadAccount { identity: "indexed-selection-reopen.read-account.v1" });
pub struct AccountReadBinding;
pub type AccountScope = ApplicationQueryFieldScope<AccountKey>;
impl ApplicationQueryBinding<IndexedSelectionSchema> for AccountReadBinding {
    type Input = ReadAccount;
    type InputBinding = ReadAccountBinding;
    type Query = AccountQuery;
    type ParameterBinding = QueryParamsBinding;
    type ResultBinding = KeyRowBinding;
    type ScopeBinding = AccountScope;
    type PrincipalBinding = OperatorPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "indexed-selection-reopen.account-read-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> AccountKey {
        AccountKey::at("AccountKey")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        IndexedSelectionSchema,
        OperatorPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        OperatorPrincipalBinding::reference()
    }
}
impl ApplicationQueryIntent<IndexedSelectionSchema> for ReadAccount {
    type Binding = AccountReadBinding;
    fn parameters(&self) -> ApplicationQueryParameterSet<AccountQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> AccountScope {
        AccountScope::new(AccountKey::at("AccountKey"), self.key)
    }
}
