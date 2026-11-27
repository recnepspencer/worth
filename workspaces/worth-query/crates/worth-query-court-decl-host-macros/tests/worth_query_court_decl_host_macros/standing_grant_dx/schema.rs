//! A quality-management site: members hold standing role grants over a site.
//! No grant names a workflow, a parent, a grantor or a delegation limit; a
//! site carries the status its grants' currentness depends on.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::{
    application_identity::ApplicationIdentityNamespace,
    application_schema::{fragment::*, *},
    authentication::*,
    worth_query_identity, worth_query_identity_namespace, worth_query_native_scalar_bindings,
    worth_query_principal_binding, worth_query_structured_value_binding, WorthQueryAction,
    WorthQueryRecord, WorthQueryRelation, WorthQuerySchema,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryInvariantMutationTarget, WorthQueryOperationBodyDenial,
};

pub const QMS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("standing-grant.dx.v1");

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    schema,
    owner = "worth.standing-grant.dx",
    name = "standing_grant_dx",
    members(super::program::QmsContribution)
)]
pub struct QmsSchema;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    fragment,
    name = "standing_grant_dx",
    bindings = Codecs,
    members(Member, Mapping, Site, RoleGrant)
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
    aspects(MemberFacts),
    relations(RoleHolder),
    reference(QmsSchema, Records)
)]
pub struct Member;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(MappingFacts),
    relations(MappingMember),
    reference(QmsSchema, Records)
)]
pub struct Mapping;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(SiteFacts), reference(QmsSchema, Records))]
pub struct Site;

/// One standing role over one site.
#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(RoleGrantFacts),
    relations(GrantSite),
    reference(QmsSchema, Records)
)]
pub struct RoleGrant;

#[derive(WorthQueryRelation)]
#[worth_query(to = Member, from = Mapping, reference(QmsSchema, Records))]
pub struct MappingMember;

/// The member holding a role grant (the capability grantee).
#[derive(WorthQueryRelation)]
#[worth_query(to = RoleGrant, from = Member, reference(QmsSchema, Records))]
pub struct RoleHolder;

/// The site a role grant is standing over (the capability resource).
#[derive(WorthQueryRelation)]
#[worth_query(to = Site, from = RoleGrant, reference(QmsSchema, Records))]
pub struct GrantSite;

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = MemberFacts, fragment = Records, entity = Member)]
pub struct MemberRow {
    #[worth_query(equality, read_only)]
    pub member_id: u64,
}

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = MappingFacts, fragment = Records, entity = Mapping)]
pub struct MappingRow {
    #[worth_query(equality)]
    pub external_identity: WorthQueryExternalPrincipalIdentity,
    #[worth_query(equality)]
    pub mapping_status: WorthQueryPrincipalMappingStatus,
}

/// `status` is `active` or `inactive`; every grant over the site depends on it.
#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = SiteFacts, fragment = Records, entity = Site)]
pub struct SiteRow {
    #[worth_query(equality, unique, read_only)]
    pub site_id: String,
    #[worth_query(equality)]
    pub site_status: String,
}

/// UTC bounds serve the review role; local calendar days serve approval.
#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = RoleGrantFacts, fragment = Records, entity = RoleGrant)]
pub struct RoleGrantRow {
    #[worth_query(equality, unique)]
    pub grant_id: String,
    #[worth_query(equality)]
    pub role: String,
    #[worth_query(equality)]
    pub purpose: String,
    #[worth_query(equality)]
    pub grant_status: String,
    pub valid_from: CanonicalTimestamp,
    pub valid_until: CanonicalTimestamp,
    pub first_day: CanonicalDate,
    pub last_day: CanonicalDate,
}

worth_query_structured_value_binding!(pub BodyDenial for WorthQueryOperationBodyDenial {
    identity: worth_query_identity!(QMS, denial)
});

worth_query_principal_binding! {
    pub MemberBinding in QmsSchema, mapping Mapping => Member {
        identity: MappingFacts::external_identity(),
        status: MappingFacts::mapping_status(),
        target: MappingMember,
        principal: MemberId = MemberFacts::member_id(),
    }
    context pub Desktop {
        idempotency: u64 = worth_query_identity!(QMS, idempotency),
        denial: BodyDenial,
    }
}

/// Sets a site's status; every grant over it follows without being touched.
#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = QmsSchema, context = Desktop, namespace = QMS, member,
    input = SetSiteStatusInput, operation = SetSiteStatusOp, binding = SetSiteStatusBinding,
    delivery = SiteStatusSet, body, program, reads(Site)
)]
pub struct SetSiteStatus {
    #[worth_query(scope = SiteId)]
    pub site: String,
    pub status: String,
}

pub fn set_site_status_operation(
) -> ApplicationOperationDefinition<QmsSchema, SetSiteStatusOp, SetSiteStatus> {
    SetSiteStatusOp::reference()
        .typed_body()
        .assign_input(SiteFacts::site_status(), SetSiteStatus::status_input())
        .finish_pure_definition()
        .expect("the site status body admits")
}

/// Grants `member` a standing review role over `site` through a handler link.
#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = QmsSchema, context = Desktop, namespace = QMS, member,
    input = AssignRoleInput, operation = AssignRoleOp, binding = AssignRoleBinding,
    delivery = RoleAssigned, result = UnitApplicationValueBinding, decision = AssignDecision,
    program,
    access(
        read_entities: [Site, Member];
        read_fields: [SiteFacts::site_id(), MemberFacts::member_id()];
        writes: [
            RoleGrantFacts::grant_id(), RoleGrantFacts::role(), RoleGrantFacts::purpose(),
            RoleGrantFacts::grant_status(), RoleGrantFacts::valid_from(), RoleGrantFacts::valid_until(),
            RoleGrantFacts::first_day(), RoleGrantFacts::last_day(),
        ];
        creates: [RoleGrant];
        links: [RoleHolder, GrantSite];
    )
)]
pub struct AssignRole {
    #[worth_query(scope = SiteId)]
    pub site: String,
    pub member: u64,
    pub grant: String,
}

/// The site and member the new grant links.
pub struct AssignDecision {
    pub site: WorthQueryInvariantMutationTarget<QmsSchema, Site>,
    pub member: WorthQueryInvariantMutationTarget<QmsSchema, Member>,
}
