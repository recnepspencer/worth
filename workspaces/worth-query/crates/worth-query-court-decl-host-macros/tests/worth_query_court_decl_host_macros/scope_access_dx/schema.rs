//! A project board whose shared access is declared once with the stock scope
//! access: typed grants, capabilities and disclosure, no access strings.
use worth_query_decl::facade::{
    application_identity::ApplicationIdentityNamespace, application_schema::fragment::*,
    authentication::*, worth_query_identity, worth_query_identity_namespace,
    worth_query_native_scalar_bindings, worth_query_principal_binding,
    worth_query_structured_value_binding, WorthQueryRecord, WorthQueryRelation, WorthQuerySchema,
};
use worth_query_host::facade::{
    primary_graph::WorthQueryOperationBodyDenial, worth_query_scope_access,
};

pub const NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("scope.access.dx.v1");

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    schema,
    owner = "worth.scope_access.dx",
    name = "scope_access_dx",
    members(super::program::BoardContribution)
)]
pub struct BoardSchema;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(fragment, name = "scope_access_dx", bindings = Codecs, members(Owner, Mapping, Project))]
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
#[worth_query(entity, aspects(OwnerFacts), reference(BoardSchema, Records))]
pub struct Owner;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(MappingFacts),
    relations(MappingOwner),
    reference(BoardSchema, Records)
)]
pub struct Mapping;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(ProjectFacts), reference(BoardSchema, Records))]
pub struct Project;

#[derive(WorthQueryRelation)]
#[worth_query(to = Owner, from = Mapping, reference(BoardSchema, Records))]
pub struct MappingOwner;

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
#[worth_query(bindings = Codecs, aspect = ProjectFacts, fragment = Records, entity = Project)]
pub struct ProjectRow {
    #[worth_query(equality, unique, read_only)]
    pub project_id: String,
    pub name: String,
}

worth_query_structured_value_binding!(pub UnitResult for () { identity: worth_query_identity!(NS, result) });
worth_query_structured_value_binding!(pub BodyDenial for WorthQueryOperationBodyDenial {
    identity: worth_query_identity!(NS, denial)
});

worth_query_principal_binding! {
    pub OwnerBinding in BoardSchema, mapping Mapping => Owner {
        identity: MappingFacts::external_identity(),
        status: MappingFacts::mapping_status(),
        target: MappingOwner,
        principal: OwnerId = OwnerFacts::owner_id(),
    }
    context pub Desktop {
        idempotency: u64 = worth_query_identity!(NS, idempotency),
        denial: BodyDenial,
    }
}

worth_query_scope_access! {
    pub mod project_access for BoardSchema {
        context: Desktop,
        scope: Project => ProjectId = ProjectFacts::project_id(),
        principal: Owner => OwnerFacts::owner_id(),
        mapping: Mapping {
            identity: MappingFacts::external_identity(),
            status: MappingFacts::mapping_status(),
        } => MappingOwner,
        reads: [ProjectFacts::project_id(), ProjectFacts::name()],
    }
}
