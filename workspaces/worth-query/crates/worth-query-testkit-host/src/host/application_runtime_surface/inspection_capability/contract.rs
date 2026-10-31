//! Complete ordinary capability contract; no implicit diagnostic grant.
use super::*;
use worth_query_decl::facade::application_schema::{
    ApplicationAuthorizationPathBuilder, ApplicationSchemaDeclarationBuilder,
};
pub(in super::super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<HostIdentitySchema>,
) -> ApplicationSchemaDeclarationBuilder<HostIdentitySchema> {
    schema
        .entity(InspectionGrant::reference())
        .aspect(
            InspectionGrant::reference(),
            InspectionGrantFacts::reference(),
        )
        .field(InspectionGrant::reference(), GrantIdentity::reference())
        .field(InspectionGrant::reference(), GrantAction::reference())
        .field(InspectionGrant::reference(), GrantPurpose::reference())
        .field(InspectionGrant::reference(), GrantDisclosure::reference())
        .field(InspectionGrant::reference(), GrantActive::reference())
        .field(InspectionGrant::reference(), GrantWorkflow::reference())
        .field(InspectionGrant::reference(), GrantNotBefore::reference())
        .field(InspectionGrant::reference(), GrantNotAfter::reference())
        .field(
            InspectionGrant::reference(),
            GrantDelegationLimit::reference(),
        )
        .relation(
            Grantor::reference(),
            Principal::reference(),
            InspectionGrant::reference(),
        )
        .relation(
            Grantee::reference(),
            Principal::reference(),
            InspectionGrant::reference(),
        )
        .relation(
            GrantResource::reference(),
            InspectionGrant::reference(),
            Account::reference(),
        )
        .relation(
            GrantParent::reference(),
            InspectionGrant::reference(),
            InspectionGrant::reference(),
        )
        .capability_context(InspectionContext::reference())
        .capability_provenance(InspectionProvenance::reference())
        .operation(
            InspectAccountOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(InspectAccountOperation::reference(), 32)
        .operation_read_field(
            InspectAccountOperation::reference(),
            AccountNumber::reference(),
        )
        .capability(definition())
}
fn definition() -> ApplicationCapabilityContract<
    HostIdentitySchema,
    InspectAccountCapability,
    InspectAccountOperation,
    String,
> {
    let allow = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(Grantor::reference())
        .forward(GrantResource::reference())
        .allow(Account::reference());
    ApplicationCapabilityContractBuilder::new(
        InspectAccountCapability::reference(),
        InspectAccountOperation::reference(),
        InspectionGrant::reference(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(GrantAction::reference(), text("inspect")),
        ApplicationCapabilityRelationBinding::from_reference(GrantResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::bound(GrantDisclosure::reference()),
        ApplicationCapabilityValueBinding::new(GrantPurpose::reference(), text("audit")),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                GrantActive::reference(),
                ApplicationEncodedScalarValue::<BoolApplicationValueBinding>::try_new(true)
                    .unwrap(),
            ),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(GrantWorkflow::reference()),
                ApplicationCapabilityFieldBinding::from_reference(AccountNumber::reference()),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(GrantNotBefore::reference()),
                ApplicationCapabilityFieldBinding::from_reference(GrantNotAfter::reference()),
            ),
        ),
        InspectionContext::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(GrantParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(Grantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(Grantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(GrantDelegationLimit::reference()),
        InspectionProvenance::reference(),
    ))
    .composition(ApplicationCapabilityComposition::new(
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
            ApplicationCapabilityDisclosureRule::permit([
                ApplicationCapabilityScopeGuard::requiring([
                    ApplicationCapabilityAcceptedValues::one_of(
                        GrantDisclosure::reference(),
                        [text("number")],
                    ),
                ]),
            ]),
        ),
    ))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}
