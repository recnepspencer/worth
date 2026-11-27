//! A role whose target is not fixed: a countersignature names an amount the
//! grant's limit must cover (a magnitude dimension), so only a request can
//! ask it.
//! An authority inventory reports it `RequiresRequest`; a capability
//! question carrying the request answers it.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::application_capability::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{
    worth_query_capability, worth_query_operation, worth_query_structured_value_binding,
};

use super::grants::{site_roles, ACTIVE};
use super::schema::*;

pub const COUNTERSIGN: &str = "countersign";

/// The site and the amount countersigned there.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CountersignRequest {
    pub site: String,
    pub amount: u64,
}

worth_query_structured_value_binding!(pub CountersignInput for CountersignRequest {
    identity: "standing-grant.dx.countersign.input.v1"
});
worth_query_capability!(pub CountersignCapability in QmsSchema);
worth_query_operation!(pub CountersignOp for QmsSchema, input CountersignInput);

impl ApplicationCapabilityRequest<QmsSchema, CountersignCapability> for CountersignRequest {
    type Scope = Site;
    type Context = site_roles::SiteContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<QmsSchema, Site, site_roles::SiteContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        let amount =
            application_capability_request_value(&RoleGrantFacts::amount_limit(), self.amount)?;
        Ok(ApplicationCapabilityRequestProjection::new(
            application_capability_selector(SiteFacts::site_id(), &*self.site)?,
            application_capability_request_value(&RoleGrantFacts::role(), COUNTERSIGN)?,
            application_capability_request_value(
                &RoleGrantFacts::purpose(),
                GrantPurpose::Quality,
            )?,
            ApplicationCapabilityRequestContext::new(site_roles::SiteContext::reference()),
        )
        .magnitude(amount))
    }
}

fn contract() -> Result<
    ApplicationCapabilityContract<
        QmsSchema,
        CountersignCapability,
        CountersignOp,
        CountersignRequest,
    >,
    ApplicationSchemaDeclarationDenial,
> {
    let any =
        |path| ApplicationCapabilityGraphRule::any([ApplicationCapabilityGraphClause::new(path)]);
    let composition = ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(any(site_roles::through_grant())),
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
    );
    let currentness = ApplicationCapabilityCurrentnessDefinition::standing(
        application_capability_grant_value(RoleGrantFacts::grant_status(), ACTIVE)?,
        ApplicationCapabilityValidityDefinition::unbounded(),
    );
    Ok(ApplicationCapabilityContractBuilder::new(
        CountersignCapability::reference(),
        CountersignOp::reference(),
        RoleGrant::reference(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        application_capability_grant_value(RoleGrantFacts::role(), COUNTERSIGN)?,
        ApplicationCapabilityRelationBinding::from_reference(GrantSite::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::not_applicable(),
        application_capability_grant_value(RoleGrantFacts::purpose(), GrantPurpose::Quality)?,
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::bound(RoleGrantFacts::amount_limit())?,
        ApplicationCapabilityCardinalityDimension::One,
        currentness,
        site_roles::SiteContext::reference(),
    ))
    .delegation(
        ApplicationCapabilityDelegationDefinition::standing(
            ApplicationCapabilityRelationBinding::from_reference(RoleHolder::reference()),
            site_roles::SiteRoleProvenance::reference(),
        )
        .forbid_self_grant(),
    )
    .composition(composition)
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build())
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<QmsSchema>,
) -> ApplicationSchemaDeclarationBuilder<QmsSchema> {
    let operation = CountersignOp::reference();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation, 64)
        .operation_read_field(operation, SiteFacts::site_id())
        .capability(contract().expect("the countersign contract declares"))
}
