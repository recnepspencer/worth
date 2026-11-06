//! Complete ordinary capability contract; no implicit diagnostic grant.
use super::*;
use worth_query_decl::facade::application_schema::{
    fragment::Registration, ApplicationSchemaDeclarationDenial,
};
use worth_query_decl::facade::application_schema::{
    ApplicationAuthorizationPathBuilder, ApplicationSchemaDeclarationBuilder,
};
impl Registration<HostIdentitySchema> for InspectionContext {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<HostIdentitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<HostIdentitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema.capability_context(Self::reference()))
    }
}
impl Registration<HostIdentitySchema> for InspectionProvenance {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<HostIdentitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<HostIdentitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema.capability_provenance(Self::reference()))
    }
}
impl Registration<HostIdentitySchema> for InspectAccountOperation {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<HostIdentitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<HostIdentitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .operation(
                Self::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(Self::reference(), 32)
            .operation_read_field(Self::reference(), AccountNumber::at("AccountNumber")))
    }
}
impl Registration<HostIdentitySchema> for InspectAccountCapability {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<HostIdentitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<HostIdentitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema.capability(definition()))
    }
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
        ApplicationCapabilityValueBinding::new(GrantAction::at("GrantAction"), text("inspect"))
            .unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(GrantResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::bound(GrantDisclosure::at("GrantDisclosure")).unwrap(),
        ApplicationCapabilityValueBinding::new(GrantPurpose::at("GrantPurpose"), text("audit"))
            .unwrap(),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                GrantActive::at("GrantActive"),
                ApplicationEncodedScalarValue::<BoolApplicationValueBinding>::try_new(true)
                    .unwrap(),
            )
            .unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(GrantWorkflow::at(
                    "GrantWorkflow",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(AccountNumber::at(
                    "AccountNumber",
                ))
                .unwrap(),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(GrantNotBefore::at(
                    "GrantNotBefore",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(GrantNotAfter::at(
                    "GrantNotAfter",
                ))
                .unwrap(),
            ),
        ),
        InspectionContext::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(GrantParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(Grantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(Grantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(GrantDelegationLimit::at(
            "GrantDelegationLimit",
        ))
        .unwrap(),
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
                        GrantDisclosure::at("GrantDisclosure"),
                        [text("number")],
                    )
                    .unwrap(),
                ]),
            ]),
        ),
    ))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}
