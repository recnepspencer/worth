//! The company capabilities: authoring and the creator's own
//! `OpenRequestCapability`, plus the subject-scoped read and instance-step
//! contracts `disclosure` and `subject_control` declare through
//! `scoped_contract`. Each is admitted
//! through an active grant whose grantee is the principal and whose resource
//! is the requested company; no capability is forged or relaxed.
use worth_query_decl::facade::application_capability::*;
use worth_query_decl::facade::application_schema::{fragment::RecordSelection, *};
use worth_query_decl::facade::{
    worth_query_capability, worth_query_capability_context, worth_query_capability_provenance,
};

use super::control::{AuthorOp, CompanyControl};
use super::creator::{OpenRequest, OpenRequestOp};
use super::schema::*;

pub const AUTHOR: &str = "author-request-workflow";
pub const START: &str = "start-request-workflow";
pub const REVIEW: &str = "review-request-workflow";
pub const ADVANCE: &str = "advance-request-workflow";
pub const OPEN: &str = "open-purchase-request";
pub const READ: &str = "read-request-workflow";
pub const ENUMERATE: &str = "enumerate-request-workflow";
const PURPOSE: &str = "purchase-request";
const ACTIVE: &str = "active";

worth_query_capability_context!(pub CompanyWorkflowContext in BirthSchema);
worth_query_capability_provenance!(pub CompanyWorkflowProvenance in BirthSchema);
worth_query_capability!(pub AuthorCapability in BirthSchema);
worth_query_capability!(pub OpenRequestCapability in BirthSchema);

pub type Projection =
    ApplicationCapabilityRequestProjection<BirthSchema, Company, CompanyWorkflowContext>;

pub fn encoded(value: &str) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value.to_owned()).expect("grant text encodes")
}

/// The company-scoped request every capability of this workflow projects.
pub fn request(
    company: &str,
    action: &str,
) -> Result<Projection, ApplicationCapabilityRequestProjectionDenial> {
    let selector =
        ApplicationCapabilityEntitySelector::new(CompanyFacts::company_id(), encoded(company))
            .map_err(|_| ApplicationCapabilityRequestProjectionDenial::input_variant("company"))?;
    Ok(ApplicationCapabilityRequestProjection::new(
        selector,
        encoded(action),
        encoded(PURPOSE),
        ApplicationCapabilityRequestContext::new(CompanyWorkflowContext::reference()),
    ))
}

pub fn field(field: impl RecordSelection) -> ApplicationCapabilityFieldBinding {
    ApplicationCapabilityFieldBinding::from_reference(field).expect("grant field binds")
}

fn value(
    field: impl RecordSelection<Value = String, Binding = StringApplicationValueBinding> + 'static,
    accepted: &str,
) -> ApplicationCapabilityValueBinding {
    ApplicationCapabilityValueBinding::new(field, encoded(accepted)).expect("grant value binds")
}

/// The principal reaches a company only through a grant naming it grantee.
fn company_path() -> ApplicationCapabilityGraphClause {
    ApplicationCapabilityGraphClause::new(
        ApplicationAuthorizationPathBuilder::from_principal(Owner::reference())
            .forward(Grantee::reference())
            .forward(GrantResource::reference())
            .allow(Company::reference()),
    )
}

fn composition(allow: ApplicationCapabilityGraphClause) -> ApplicationCapabilityComposition {
    ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(ApplicationCapabilityGraphRule::any([allow])),
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

/// A company-scoped contract over `action`.
pub fn contract<Cap, Op, Input>(
    capability: ApplicationCapabilityRef<BirthSchema, Cap>,
    operation: ApplicationOperationRef<BirthSchema, Op, Input>,
    action: &str,
) -> ApplicationCapabilityContract<BirthSchema, Cap, Op, Input>
where
    Cap: ApplicationCapabilityMarkerIdentity<Schema = BirthSchema> + 'static,
    Op: ApplicationOperationMarkerIdentity<BirthSchema> + 'static,
    Op::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
    Input: ApplicationCapabilityRequest<BirthSchema, Cap> + 'static,
{
    let resource = ApplicationCapabilityRelationBinding::from_reference(GrantResource::reference());
    let company = field(CompanyFacts::company_id());
    scoped_contract(
        capability,
        operation,
        action,
        company_path(),
        resource,
        company,
    )
}

/// A grant of `action` reaching the requested resource through `allow`;
/// the grant's workflow names the resource's `workflow` field.
pub fn scoped_contract<Cap, Op, Input>(
    capability: ApplicationCapabilityRef<BirthSchema, Cap>,
    operation: ApplicationOperationRef<BirthSchema, Op, Input>,
    action: &str,
    allow: ApplicationCapabilityGraphClause,
    resource: impl Into<ApplicationCapabilityResourcePath>,
    workflow: ApplicationCapabilityFieldBinding,
) -> ApplicationCapabilityContract<BirthSchema, Cap, Op, Input>
where
    Cap: ApplicationCapabilityMarkerIdentity<Schema = BirthSchema> + 'static,
    Op: ApplicationOperationMarkerIdentity<BirthSchema> + 'static,
    Op::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
    Input: ApplicationCapabilityRequest<BirthSchema, Cap> + 'static,
{
    ApplicationCapabilityContractBuilder::new(capability, operation, Grant::reference())
        .target(ApplicationCapabilityTargetDefinition::new(
            value(GrantFacts::action(), action),
            resource,
            ApplicationCapabilityRelationDimension::not_applicable(),
            ApplicationCapabilityFieldDimension::not_applicable(),
            value(GrantFacts::purpose(), PURPOSE),
        ))
        .constraints(ApplicationCapabilityConstraintDefinition::new(
            ApplicationCapabilityMagnitudeDimension::not_applicable(),
            ApplicationCapabilityCardinalityDimension::One,
            ApplicationCapabilityCurrentnessDefinition::new(
                value(GrantFacts::status(), ACTIVE),
                ApplicationCapabilityWorkflowDefinition::new(
                    field(GrantFacts::workflow()),
                    workflow,
                ),
                ApplicationCapabilityValidityDefinition::new(
                    ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                    field(GrantFacts::not_before()),
                    field(GrantFacts::not_after()),
                ),
            ),
            CompanyWorkflowContext::reference(),
        ))
        .delegation(ApplicationCapabilityDelegationDefinition::new(
            ApplicationCapabilityRelationBinding::from_reference(GrantParent::reference()),
            ApplicationCapabilityRelationBinding::from_reference(Grantor::reference()),
            ApplicationCapabilityRelationBinding::from_reference(Grantee::reference()),
            field(GrantFacts::delegation_limit()),
            CompanyWorkflowProvenance::reference(),
        ))
        .composition(composition(allow))
        .elevation(ApplicationCapabilityElevationRule::not_applicable())
        .build()
}

/// Declares the context, provenance and the four contracts, after the
/// control operations and the creator's body operation are declared.
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<BirthSchema>,
) -> ApplicationSchemaDeclarationBuilder<BirthSchema> {
    schema
        .capability_context(CompanyWorkflowContext::reference())
        .capability_provenance(CompanyWorkflowProvenance::reference())
        .capability(contract::<_, _, CompanyControl>(
            AuthorCapability::reference(),
            AuthorOp::reference(),
            AUTHOR,
        ))
        .capability(contract::<_, _, OpenRequest>(
            OpenRequestCapability::reference(),
            OpenRequestOp::reference(),
            OPEN,
        ))
}
