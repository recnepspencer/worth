//! A QMS-shaped case board: users hold role assignments at sites, cases belong
//! to a site and name exactly one author.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::{
    application_identity::ApplicationIdentityNamespace,
    application_schema::{fragment::*, *},
    authentication::*,
    worth_query_identity, worth_query_identity_namespace, worth_query_native_scalar_bindings,
    worth_query_operation_requires, worth_query_policy, worth_query_principal_binding,
    worth_query_structured_value_binding, WorthQuery, WorthQueryAbilities, WorthQueryAction,
    WorthQueryRecord, WorthQueryRelation, WorthQueryRow, WorthQuerySchema, WorthQueryVocabulary,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDenial;

pub const REBAC: ApplicationIdentityNamespace = worth_query_identity_namespace!("rebac.dx.v1");

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    schema,
    owner = "worth.rebac.dx",
    name = "rebac_dx",
    members(super::program::RebacContribution)
)]
pub struct RebacSchema;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    fragment,
    name = "rebac_dx",
    bindings = Codecs,
    members(User, Mapping, Site, Assignment, Case)
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

/// Durable role codes, stored through the derived identity binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq, WorthQueryVocabulary)]
#[worth_query(identity = "rebac.dx.role.v1", bindings = Codecs)]
pub enum StaffRole {
    Reviewer = 1,
    Manager = 2,
    Auditor = 3,
}

/// Everything a case-scoped principal may do, one ability marker each.
#[derive(Clone, Copy, Debug, Eq, PartialEq, WorthQueryAbilities)]
#[worth_query(schema = RebacSchema, scoped_to = Case)]
pub enum CaseAbility {
    Annotate,
    Close,
    ReadCase,
}

impl ApplicationRoleMatrix<CaseAbility> for StaffRole {
    fn abilities(self) -> &'static [CaseAbility] {
        use CaseAbility::*;
        match self {
            Self::Reviewer => &[Annotate, ReadCase],
            Self::Manager => &[Annotate, Close, ReadCase],
            Self::Auditor => &[ReadCase],
        }
    }
}

worth_query_policy!(pub CaseAccess in RebacSchema);

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(UserFacts),
    relations(UserAssignments),
    reference(RebacSchema, Records)
)]
pub struct User;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(MappingFacts),
    relations(MappingUser),
    reference(RebacSchema, Records)
)]
pub struct Mapping;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(SiteFacts), reference(RebacSchema, Records))]
pub struct Site;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(AssignmentFacts),
    relations(AssignmentSite),
    reference(RebacSchema, Records)
)]
pub struct Assignment;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(CaseFacts),
    relations(CaseSite, CaseAuthor),
    reference(RebacSchema, Records)
)]
pub struct Case;

#[derive(WorthQueryRelation)]
#[worth_query(to = User, from = Mapping, reference(RebacSchema, Records))]
pub struct MappingUser;

#[derive(WorthQueryRelation)]
#[worth_query(to = Assignment, from = User, reference(RebacSchema, Records))]
pub struct UserAssignments;

#[derive(WorthQueryRelation)]
#[worth_query(to = Site, from = Assignment, reference(RebacSchema, Records))]
pub struct AssignmentSite;

#[derive(WorthQueryRelation)]
#[worth_query(to = Site, from = Case, reference(RebacSchema, Records))]
pub struct CaseSite;

/// Every live case names exactly one author, so `actor_is_not_target_of`
/// can deny the author without failing open on a missing edge.
#[derive(WorthQueryRelation)]
#[worth_query(
    to = User,
    from = Case,
    reference(RebacSchema, Records),
    cardinality = ApplicationRelationCardinality::new(Some(1), Some(1), None, None, None, None)
)]
pub struct CaseAuthor;

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = UserFacts, fragment = Records, entity = User)]
pub struct UserRow {
    #[worth_query(equality, read_only)]
    pub user_id: u64,
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
#[worth_query(bindings = Codecs, aspect = SiteFacts, fragment = Records, entity = Site)]
pub struct SiteRow {
    #[worth_query(equality, unique, read_only)]
    pub site_id: u64,
}

/// A role held at one site, with a qualification level.
#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = AssignmentFacts, fragment = Records, entity = Assignment)]
pub struct AssignmentRow {
    #[worth_query(equality)]
    pub role: StaffRole,
    #[worth_query(equality)]
    pub level: u64,
}

/// `author_id` records the author's user id as a field, beside the
/// `CaseAuthor` edge, so a principal-field condition can compare it.
#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = CaseFacts, fragment = Records, entity = Case)]
pub struct CaseRow {
    #[worth_query(equality, unique, read_only)]
    pub case_id: u64,
    #[worth_query(equality)]
    pub author_id: u64,
    pub note: u64,
    /// Optional: absent until a case is escalated.
    #[worth_query(equality)]
    pub escalated_by: Option<u64>,
}

worth_query_structured_value_binding!(pub BodyDenial for WorthQueryOperationBodyDenial {
    identity: worth_query_identity!(REBAC, denial)
});

worth_query_principal_binding! {
    pub UserBinding in RebacSchema, mapping Mapping => User {
        identity: MappingFacts::external_identity(),
        status: MappingFacts::mapping_status(),
        target: MappingUser,
        principal: UserId = UserFacts::user_id(),
    }
    context pub Desktop {
        idempotency: u64 = worth_query_identity!(REBAC, idempotency),
        denial: BodyDenial,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = RebacSchema, root = Case, namespace = REBAC, member)]
pub struct CaseView {
    #[worth_query(field = CaseId)]
    pub id: u64,
    #[worth_query(field = Note)]
    pub note: u64,
}

/// Reading a case requires the `ReadCase` ability on that case.
#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = RebacSchema, context = Desktop, namespace = REBAC, member,
    result = CaseView, root = Case, scope = CaseId, cardinality = ExactlyOne, limits = 1,
    ability = case_ability::ReadCase::reference(),
)]
pub struct ReadCase {
    #[worth_query(scope)]
    pub case: u64,
}

/// Writes a note on a case; requires `Annotate`.
#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = RebacSchema, context = Desktop, namespace = REBAC, member,
    input = AnnotateCaseInput, operation = AnnotateCaseOp, binding = AnnotateCaseBinding,
    delivery = CaseAnnotated, body, program, reads(Case)
)]
pub struct AnnotateCase {
    #[worth_query(scope = CaseId)]
    pub case: u64,
    pub note: u64,
}

/// Closes a case with a final note; requires `Close`.
#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = RebacSchema, context = Desktop, namespace = REBAC, member,
    input = CloseCaseInput, operation = CloseCaseOp, binding = CloseCaseBinding,
    delivery = CaseClosed, body, program, reads(Case)
)]
pub struct CloseCase {
    #[worth_query(scope = CaseId)]
    pub case: u64,
    pub note: u64,
}

worth_query_operation_requires!(AnnotateCaseOp => [case_ability::Annotate]);
worth_query_operation_requires!(CloseCaseOp => [case_ability::Close]);

pub fn annotate_operation(
) -> ApplicationOperationDefinition<RebacSchema, AnnotateCaseOp, AnnotateCase> {
    AnnotateCaseOp::reference()
        .typed_body()
        .assign_input(CaseFacts::note(), AnnotateCase::note_input())
        .finish_pure_definition()
        .expect("annotate body admits")
}

pub fn close_operation() -> ApplicationOperationDefinition<RebacSchema, CloseCaseOp, CloseCase> {
    CloseCaseOp::reference()
        .typed_body()
        .assign_input(CaseFacts::note(), CloseCase::note_input())
        .finish_pure_definition()
        .expect("close body admits")
}
