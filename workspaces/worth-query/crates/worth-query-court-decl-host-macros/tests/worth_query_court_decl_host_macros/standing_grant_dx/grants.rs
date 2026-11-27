//! Five standing site roles as one `worth_query_standing_grants!` table over
//! the RoleGrant record. No grant names a workflow, parent, grantor or
//! delegation limit; every role fails closed when its site is not active and
//! no operation may grant its own principal a role. Validity windows cover
//! every shape: UTC and local-date closed windows, `until` (open below),
//! `from` (open above) and `unbounded()`. Audit also needs an enabled site
//! assignment, a second half its eligibility must agree on.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{
    worth_query_ability, worth_query_policy, worth_query_standing_grants,
    worth_query_structured_value_binding,
};

use super::schema::*;

pub const REVIEW: &str = "review";
pub const APPROVE: &str = "approve";
pub const AUDIT: &str = "audit";
pub const WITNESS: &str = "witness";
pub const OBSERVE: &str = "observe";
pub const ACTIVE: &str = "active";
pub const ENABLED: &str = "enabled";

/// The site a role is exercised over.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SiteRequest {
    pub site: String,
}

worth_query_structured_value_binding!(pub SiteRequestInput for SiteRequest {
    identity: "standing-grant.dx.site-request.input.v1"
});

worth_query_standing_grants! {
    pub mod site_roles for QmsSchema {
        principal: Member,
        request: SiteRequest => SiteRequestInput, input.site,
        scope: Site = SiteFacts::site_id(),
        context: SiteContext,
        provenance: SiteRoleProvenance,
        grant: RoleGrant {
            holder: RoleHolder,
            resource: GrantSite,
            role: RoleGrantFacts::role(),
            purpose: RoleGrantFacts::purpose() => GrantPurpose::Quality,
            active: RoleGrantFacts::grant_status() => ACTIVE,
        },
        require_active: [SiteFacts::site_status() => ACTIVE],
        work_budget: 64,
        roles: [
            pub ReviewCapability / ReviewOp = REVIEW,
                utc(RoleGrantFacts::valid_from(), RoleGrantFacts::valid_until());
            pub ApproveCapability / ApproveOp = APPROVE,
                local_date(RoleGrantFacts::first_day(), RoleGrantFacts::last_day());
            pub AuditCapability / AuditOp = AUDIT,
                until(RoleGrantFacts::valid_until()), allow through_enabled_assignment();
            pub WitnessCapability / WitnessOp = WITNESS, from(RoleGrantFacts::first_day());
            pub ObserveCapability / ObserveOp = OBSERVE, unbounded();
        ],
    }
}
pub use site_roles::*;

// Any role holder over a site may see who could audit it.
worth_query_ability!(pub ViewAuditors scoped_to Site, in QmsSchema);
worth_query_policy!(pub AuditorRosterPolicy in QmsSchema);

/// The principal reaches a site through an enabled assignment over it.
fn through_enabled_assignment() -> ApplicationAuthorizationPath {
    let enabled = ApplicationEncodedScalarValue::try_new(ENABLED.to_owned()).expect("encodes");
    ApplicationAuthorizationPathBuilder::from_principal(Member::reference())
        .forward(AssignmentHolder::reference())
        .where_equal(AssignmentFacts::assignment_status(), enabled)
        .expect("assignment status is equality-queryable")
        .forward(AssignmentSite::reference())
        .allow(Site::reference())
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<QmsSchema>,
) -> ApplicationSchemaDeclarationBuilder<QmsSchema> {
    let schema = site_roles::members(schema).expect("the site roles declare");
    super::countersign::declare(schema)
        .ability(ViewAuditors::reference())
        .policy(AuditorRosterPolicy::reference())
        .ability_policy(
            ViewAuditors::reference(),
            AuditorRosterPolicy::reference(),
            [through_grant()],
        )
}
