//! A company whose purchase requests are born enrolled in a workflow. Each
//! request stamps `created_at` from trusted admission time; grants give the
//! owner the four workflow capabilities over the company.
use worth_query_decl::facade::{
    application_identity::ApplicationIdentityNamespace,
    application_schema::{fragment::*, *},
    authentication::*,
    worth_query_identity, worth_query_identity_namespace, worth_query_native_scalar_bindings,
    worth_query_principal_binding, worth_query_structured_value_binding, WorthQuery,
    WorthQueryRecord, WorthQueryRelation, WorthQueryRow, WorthQuerySchema,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDenial;

pub const BIRTH: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("workflow-birth-time.dx.v1");

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    schema,
    owner = "worth.workflow-birth-time.dx",
    name = "workflow_birth_time_dx",
    members(super::program::BirthContribution)
)]
pub struct BirthSchema;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    fragment,
    name = "workflow_birth_time_dx",
    bindings = Codecs,
    members(Owner, Mapping, Company, Request, Grant)
)]
pub struct Records;

worth_query_native_scalar_bindings!(pub Codecs);
impl Scalar<WorthQueryExternalPrincipalIdentity> for Codecs {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for Codecs {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(OwnerFacts),
    relations(Grantor, Grantee),
    reference(BirthSchema, Records)
)]
pub struct Owner;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(MappingFacts),
    relations(MappingOwner),
    reference(BirthSchema, Records)
)]
pub struct Mapping;

/// The container a request is created in: the creator's capability scope.
#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(CompanyFacts), reference(BirthSchema, Records))]
pub struct Company;

/// The subject born enrolled in its workflow, linked to its company at birth.
#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(RequestFacts),
    relations(RequestCompany),
    reference(BirthSchema, Records)
)]
pub struct Request;

/// One grant of one workflow capability over one company.
#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(GrantFacts),
    relations(GrantResource, GrantParent),
    reference(BirthSchema, Records)
)]
pub struct Grant;

#[derive(WorthQueryRelation)]
#[worth_query(to = Owner, from = Mapping, reference(BirthSchema, Records))]
pub struct MappingOwner;

/// The birth edge a subject-scoped read follows from a company grant.
#[derive(WorthQueryRelation)]
#[worth_query(to = Company, from = Request, reference(BirthSchema, Records))]
pub struct RequestCompany;

#[derive(WorthQueryRelation)]
#[worth_query(to = Grant, from = Owner, reference(BirthSchema, Records))]
pub struct Grantor;

#[derive(WorthQueryRelation)]
#[worth_query(to = Grant, from = Owner, reference(BirthSchema, Records))]
pub struct Grantee;

#[derive(WorthQueryRelation)]
#[worth_query(to = Company, from = Grant, reference(BirthSchema, Records))]
pub struct GrantResource;

#[derive(WorthQueryRelation)]
#[worth_query(to = Grant, from = Grant, reference(BirthSchema, Records))]
pub struct GrantParent;

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = OwnerFacts, fragment = Records, entity = Owner)]
pub struct OwnerRow {
    #[worth_query(equality, read_only)]
    pub owner_id: u64,
}

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = MappingFacts, fragment = Records, entity = Mapping)]
pub struct MappingRow {
    #[worth_query(equality)]
    pub external_identity: WorthQueryExternalPrincipalIdentity,
    #[worth_query(equality)]
    pub mapping_status: WorthQueryPrincipalMappingStatus,
}

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = CompanyFacts, fragment = Records, entity = Company)]
pub struct CompanyRow {
    #[worth_query(equality, unique, read_only)]
    pub company_id: String,
}

/// `created_at` is read-only: only the birth body initializes it, from
/// trusted admission time. No caller input carries it.
#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = RequestFacts, fragment = Records, entity = Request)]
pub struct RequestRow {
    #[worth_query(equality, unique, read_only)]
    pub request_id: u64,
    #[worth_query(read_only)]
    pub created_at: CanonicalTimestamp,
    /// Copied from the birth scope; a subject grant's workflow names it.
    #[worth_query(equality, read_only)]
    pub company_code: String,
}

/// The typed dimensions every workflow capability contract reads.
#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = GrantFacts, fragment = Records, entity = Grant)]
pub struct GrantRow {
    #[worth_query(equality, unique)]
    pub grant_id: String,
    #[worth_query(equality)]
    pub action: String,
    #[worth_query(equality)]
    pub purpose: String,
    #[worth_query(equality)]
    pub status: String,
    #[worth_query(equality)]
    pub workflow: String,
    #[worth_query(equality)]
    pub not_before: u64,
    #[worth_query(equality)]
    pub not_after: u64,
    #[worth_query(equality)]
    pub delegation_limit: u64,
}

worth_query_structured_value_binding!(pub BodyDenial for WorthQueryOperationBodyDenial {
    identity: worth_query_identity!(BIRTH, denial)
});

worth_query_principal_binding! {
    pub OwnerBinding in BirthSchema, mapping Mapping => Owner {
        identity: MappingFacts::external_identity(),
        status: MappingFacts::mapping_status(),
        target: MappingOwner,
        principal: OwnerId = OwnerFacts::owner_id(),
    }
    context pub Desktop {
        idempotency: u64 = worth_query_identity!(BIRTH, idempotency),
        denial: BodyDenial,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = BirthSchema, root = Request, namespace = BIRTH, member)]
pub struct RequestView {
    #[worth_query(field = RequestId)]
    pub id: u64,
    #[worth_query(field = CreatedAt)]
    pub created_at: CanonicalTimestamp,
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = BirthSchema, context = Desktop, namespace = BIRTH, member,
    result = RequestView, root = Request, scope = RequestId, cardinality = ExactlyOne, limits = 1,
)]
pub struct ReadRequest {
    #[worth_query(scope)]
    pub request: u64,
}
