//! A parent-scoped creation's source disclosure: `AssignRole` is scoped by
//! its site and creates a `RoleGrant`, so its disclosure query is scoped by
//! the site too and reaches the created grant through a root path
//! (`Site ← GrantSite ← RoleGrant`). Evidence reads are governed by a review
//! grant whose contract discloses quality-purpose evidence.
use worth_query_decl::facade::application_capability::*;
use worth_query_decl::facade::application_operation::{
    ApplicationSourceDisclosureBinding, ApplicationSourceDisclosureParameterDenial,
};
use worth_query_decl::facade::application_query::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{
    worth_query_capability, worth_query_disclosure_classification, worth_query_operation,
    WorthQuery, WorthQueryRow,
};

use super::grants::{site_roles, SiteRequest, SiteRequestInput, ACTIVE, REVIEW};
use super::schema::*;

worth_query_capability!(pub EvidenceCapability in QmsSchema);
worth_query_operation!(pub InspectEvidenceOp for QmsSchema, input SiteRequestInput);
worth_query_disclosure_classification!(GrantEvidence);
worth_query_disclosure_classification!(GrantEvidenceMetadata);

/// The disclosure value evidence rules and the evidence request share.
fn quality() -> ApplicationEncodedScalarValue<<Purpose as fragment::RecordSelection>::Binding> {
    ApplicationEncodedScalarValue::try_new(GrantPurpose::Quality).expect("the purpose encodes")
}

impl ApplicationCapabilityRequest<QmsSchema, EvidenceCapability> for SiteRequest {
    type Scope = Site;
    type Context = site_roles::SiteContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<QmsSchema, Site, site_roles::SiteContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            application_capability_selector(SiteFacts::site_id(), &*self.site)?,
            application_capability_request_value(&RoleGrantFacts::role(), REVIEW)?,
            application_capability_request_value(
                &RoleGrantFacts::purpose(),
                GrantPurpose::Quality,
            )?,
            ApplicationCapabilityRequestContext::new(site_roles::SiteContext::reference()),
        )
        .field(quality()))
    }
}

/// The grant a site's assignment created. Only inspection reads this
/// projection; no court runs the query for rows.
#[derive(Clone, Debug, WorthQueryRow)]
#[expect(
    dead_code,
    reason = "the disclosure query's row is read by inspection only"
)]
#[worth_query(schema = QmsSchema, root = RoleGrant, namespace = QMS, member)]
pub struct AssignedGrantRow {
    #[worth_query(field = GrantId)]
    pub grant_id: String,
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(schema = QmsSchema, context = Desktop, namespace = QMS, member,
    result = AssignedGrantRow, root = RoleGrant, limits = 1)]
pub struct AssignedGrant {
    #[worth_query(scope = SiteId)]
    pub site: String,
    pub grant: String,
}

/// `AssignRole`'s inspection disclosure: the grant it created, on its site.
pub struct AssignRoleProvenance;

impl ApplicationSourceDisclosureBinding<QmsSchema> for AssignRoleProvenance {
    const IDENTITY: &'static str = "standing-grant.dx.assign-role.provenance.v1";
    type Source = AssignRoleBinding;
    type Disclosure = AssignedGrantBinding;
    fn disclosure_parameters(
        original: &AssignRole,
    ) -> Result<
        ApplicationQueryParameterSet<AssignedGrantQuery>,
        ApplicationSourceDisclosureParameterDenial,
    > {
        Ok(AssignedGrant {
            site: original.site.clone(),
            grant: original.grant.clone(),
        }
        .parameters())
    }
}

/// Grant-level evidence governance: the grant id is used and disclosed to a
/// quality-purpose evidence grant, and so is every inspection metadata facet.
fn assigned_grant_query(
) -> ApplicationQueryDefinitionBuilder<QmsSchema, AssignedGrantQuery, (), AssignedGrantRow, Site> {
    let influence = ApplicationQueryInfluenceContract::permit_all();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        GrantEvidence,
        EvidenceCapability::reference(),
    )
    .use_field_by(RoleGrantFacts::grant_id(), quality(), influence.clone())
    .expect("the grant id is governed")
    .disclose_field_by(
        AssignedGrantRow::grant_id::<AssignedGrantQuery>(),
        quality(),
        influence,
    );
    ApplicationQueryDefinitionBuilder::declare(AssignedGrantQuery::reference())
        .root(RoleGrant::reference())
        .scope(Site::reference())
        .result_shape(AssignedGrantRow::result_shape_builder().build())
        .cardinality(ApplicationQueryCardinality::OptionalOne)
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .parameter(AssignedGrantQuery::grant())
        .where_equal(RoleGrantFacts::grant_id(), AssignedGrantQuery::grant())
        .root_path(
            ApplicationQueryRootPath::from(Site::reference()).reverse(GrantSite::reference()),
        )
}

fn inspection() -> ApplicationQueryInspectionDefinition<QmsSchema, AssignedGrantBinding> {
    let mut metadata = ApplicationQueryDisclosureContract::governed_by(
        GrantEvidenceMetadata,
        EvidenceCapability::reference(),
    );
    for facet in ApplicationQueryInspectionMetadataFacet::ALL {
        metadata = metadata.disclose_inspection_metadata_by(
            ApplicationQueryInspectionMetadataRef::<QmsSchema, AssignedGrantBinding>::new(facet),
            quality(),
            ApplicationQueryInfluenceContract::permit([facet.influence()]),
        );
    }
    ApplicationQueryInspectionDefinition::new(metadata).expect("the evidence metadata declares")
}

/// A review grant over the site, standing and unbounded, whose contract
/// discloses quality-purpose evidence.
fn evidence_contract(
) -> ApplicationCapabilityContract<QmsSchema, EvidenceCapability, InspectEvidenceOp, SiteRequest> {
    let grant = |field, value| application_capability_grant_value(field, value);
    let disclose =
        ApplicationCapabilityDisclosureRule::permit([ApplicationCapabilityScopeGuard::requiring(
            [
                ApplicationCapabilityAcceptedValues::one_of(RoleGrantFacts::purpose(), [quality()])
                    .expect("the purpose is accepted"),
            ],
        )]);
    let any =
        |path| ApplicationCapabilityGraphRule::any([ApplicationCapabilityGraphClause::new(path)]);
    ApplicationCapabilityContractBuilder::new(
        EvidenceCapability::reference(),
        InspectEvidenceOp::reference(),
        RoleGrant::reference(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        grant(RoleGrantFacts::role(), REVIEW.to_owned()).expect("the role encodes"),
        ApplicationCapabilityRelationBinding::from_reference(GrantSite::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::bound(RoleGrantFacts::purpose())
            .expect("the purpose bounds the field"),
        application_capability_grant_value(RoleGrantFacts::purpose(), GrantPurpose::Quality)
            .expect("the purpose encodes"),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::standing(
            grant(RoleGrantFacts::grant_status(), ACTIVE.to_owned()).expect("the status encodes"),
            ApplicationCapabilityValidityDefinition::unbounded(),
        ),
        site_roles::SiteContext::reference(),
    ))
    .delegation(
        ApplicationCapabilityDelegationDefinition::standing(
            ApplicationCapabilityRelationBinding::from_reference(RoleHolder::reference()),
            site_roles::SiteRoleProvenance::reference(),
        )
        .forbid_self_grant(),
    )
    .composition(ApplicationCapabilityComposition::new(
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
            disclose,
        ),
    ))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}

/// The evidence operation, its contract, the disclosure query and its
/// inspection metadata.
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<QmsSchema>,
) -> ApplicationSchemaDeclarationBuilder<QmsSchema> {
    let operation = InspectEvidenceOp::reference();
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
        .capability(evidence_contract())
        .application_query(assigned_grant_query())
        .application_query_binding::<AssignedGrantBinding>()
        .application_query_inspection(inspection())
}
