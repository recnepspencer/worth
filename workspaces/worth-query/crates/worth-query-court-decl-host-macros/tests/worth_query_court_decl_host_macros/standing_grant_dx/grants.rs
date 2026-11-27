//! Two standing site roles, authored with no workflow, lineage or shadow
//! fields: review is valid between UTC instants and approval between local
//! calendar days. Both fail closed when their site is not active, and no
//! operation may grant its own principal a role.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::application_capability::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{
    worth_query_capability, worth_query_capability_context, worth_query_capability_provenance,
    worth_query_operation, worth_query_structured_value_binding,
};

use super::schema::*;

pub const REVIEW: &str = "review";
pub const APPROVE: &str = "approve";
pub const PURPOSE: &str = "quality";
pub const ACTIVE: &str = "active";

worth_query_capability_context!(pub SiteContext in QmsSchema);
worth_query_capability_provenance!(pub SiteRoleProvenance in QmsSchema);
worth_query_capability!(pub ReviewCapability in QmsSchema);
worth_query_capability!(pub ApproveCapability in QmsSchema);

/// The site a role is exercised over.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SiteRequest {
    pub site: String,
}

worth_query_structured_value_binding!(pub SiteRequestInput for SiteRequest {
    identity: "standing-grant.dx.site-request.input.v1"
});
worth_query_operation!(pub ReviewOp for QmsSchema, input SiteRequestInput);
worth_query_operation!(pub ApproveOp for QmsSchema, input SiteRequestInput);

fn text(value: &str) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value.to_owned()).expect("role text encodes")
}

fn value(
    field: impl fragment::RecordSelection<Value = String, Binding = StringApplicationValueBinding>
        + 'static,
    accepted: &str,
) -> ApplicationCapabilityValueBinding {
    ApplicationCapabilityValueBinding::new(field, text(accepted)).expect("role value binds")
}

fn projection(
    request: &SiteRequest,
    role: &str,
) -> Result<
    ApplicationCapabilityRequestProjection<QmsSchema, Site, SiteContext>,
    ApplicationCapabilityRequestProjectionDenial,
> {
    let site = ApplicationCapabilityEntitySelector::new(SiteFacts::site_id(), text(&request.site))
        .map_err(|_| ApplicationCapabilityRequestProjectionDenial::input_variant("site"))?;
    Ok(ApplicationCapabilityRequestProjection::new(
        site,
        text(role),
        text(PURPOSE),
        ApplicationCapabilityRequestContext::new(SiteContext::reference()),
    ))
}

impl ApplicationCapabilityRequest<QmsSchema, ReviewCapability> for SiteRequest {
    type Scope = Site;
    type Context = SiteContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<QmsSchema, Site, SiteContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        projection(self, REVIEW)
    }
}

impl ApplicationCapabilityRequest<QmsSchema, ApproveCapability> for SiteRequest {
    type Scope = Site;
    type Context = SiteContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<QmsSchema, Site, SiteContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        projection(self, APPROVE)
    }
}

/// The principal reaches a site only through a role grant it holds.
fn composition() -> ApplicationCapabilityComposition {
    let allow = ApplicationAuthorizationPathBuilder::from_principal(Member::reference())
        .forward(RoleHolder::reference())
        .forward(GrantSite::reference())
        .allow(Site::reference());
    ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(ApplicationCapabilityGraphRule::any([
                ApplicationCapabilityGraphClause::new(allow),
            ])),
            ApplicationCapabilityDenyRule::not_applicable(),
            ApplicationCapabilityConflictRule::not_applicable(),
        ),
        ApplicationCapabilityActorComposition::new(
            ApplicationCapabilitySeparationOfDutyRule::not_applicable(),
            ApplicationCapabilityDistinctActorRule::not_applicable(),
            ApplicationCapabilityRecordedActorRule::not_applicable(),
        ),
        ApplicationCapabilityPropagationComposition::new(
            ApplicationCapabilityDelegationRule::forbidden(),
            ApplicationCapabilityDisclosureRule::not_applicable(),
        ),
    )
}

/// One standing site role: current while the grant is active, inside its
/// validity and its site is active. That is the whole currentness story.
fn standing_role<Cap, Op>(
    capability: ApplicationCapabilityRef<QmsSchema, Cap>,
    operation: ApplicationOperationRef<QmsSchema, Op, SiteRequest>,
    role: &str,
    validity: ApplicationCapabilityValidityDefinition,
) -> ApplicationCapabilityContract<QmsSchema, Cap, Op, SiteRequest>
where
    Cap: ApplicationCapabilityMarkerIdentity<Schema = QmsSchema> + 'static,
    Op: ApplicationOperationMarkerIdentity<QmsSchema, InputBinding = SiteRequestInput> + 'static,
    SiteRequest: ApplicationCapabilityRequest<QmsSchema, Cap>,
{
    ApplicationCapabilityContractBuilder::new(capability, operation, RoleGrant::reference())
        .target(ApplicationCapabilityTargetDefinition::new(
            value(RoleGrantFacts::role(), role),
            ApplicationCapabilityRelationBinding::from_reference(GrantSite::reference()),
            ApplicationCapabilityRelationDimension::not_applicable(),
            ApplicationCapabilityFieldDimension::not_applicable(),
            value(RoleGrantFacts::purpose(), PURPOSE),
        ))
        .constraints(ApplicationCapabilityConstraintDefinition::new(
            ApplicationCapabilityMagnitudeDimension::not_applicable(),
            ApplicationCapabilityCardinalityDimension::One,
            ApplicationCapabilityCurrentnessDefinition::standing(
                value(RoleGrantFacts::grant_status(), ACTIVE),
                validity,
            )
            .require_active(value(SiteFacts::site_status(), ACTIVE)),
            SiteContext::reference(),
        ))
        .delegation(
            ApplicationCapabilityDelegationDefinition::standing(
                ApplicationCapabilityRelationBinding::from_reference(RoleHolder::reference()),
                SiteRoleProvenance::reference(),
            )
            .forbid_self_grant(),
        )
        .composition(composition())
        .elevation(ApplicationCapabilityElevationRule::not_applicable())
        .build()
}

fn pure<Op>(
    operation: ApplicationOperationRef<QmsSchema, Op, SiteRequest>,
) -> ApplicationOperationDefinition<QmsSchema, Op, SiteRequest>
where
    Op: ApplicationOperationMarkerIdentity<QmsSchema, InputBinding = SiteRequestInput>,
{
    operation
        .definition()
        .no_external_effect()
        .no_aftermath()
        .finish()
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<QmsSchema>,
) -> ApplicationSchemaDeclarationBuilder<QmsSchema> {
    let (review, approve) = (ReviewOp::reference(), ApproveOp::reference());
    schema
        .capability_context(SiteContext::reference())
        .capability_provenance(SiteRoleProvenance::reference())
        .operation(pure(review))
        .operation_projection_work_budget(review, 64)
        .operation_read_field(review, SiteFacts::site_id())
        .operation(pure(approve))
        .operation_projection_work_budget(approve, 64)
        .operation_read_field(approve, SiteFacts::site_id())
        .capability(standing_role(
            ReviewCapability::reference(),
            review,
            REVIEW,
            ApplicationCapabilityValidityDefinition::utc(
                RoleGrantFacts::valid_from(),
                RoleGrantFacts::valid_until(),
            )
            .expect("UTC validity binds"),
        ))
        .capability(standing_role(
            ApproveCapability::reference(),
            approve,
            APPROVE,
            ApplicationCapabilityValidityDefinition::local_date(
                RoleGrantFacts::first_day(),
                RoleGrantFacts::last_day(),
            )
            .expect("local-date validity binds"),
        ))
}
