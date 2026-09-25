use worth_query_host::facade::declaration::{
    application_capability::{
        ApplicationCapabilityActorComposition, ApplicationCapabilityAllowRule,
        ApplicationCapabilityCardinalityDimension, ApplicationCapabilityComposition,
        ApplicationCapabilityConflictRule, ApplicationCapabilityConstraintDefinition,
        ApplicationCapabilityContract, ApplicationCapabilityContractBuilder,
        ApplicationCapabilityCurrentnessDefinition, ApplicationCapabilityDecisionComposition,
        ApplicationCapabilityDelegationDefinition, ApplicationCapabilityDelegationRule,
        ApplicationCapabilityDenyRule, ApplicationCapabilityDisclosureRule,
        ApplicationCapabilityDistinctActorRule, ApplicationCapabilityElevationRule,
        ApplicationCapabilityFieldBinding, ApplicationCapabilityFieldDimension,
        ApplicationCapabilityGraphClause, ApplicationCapabilityGraphRule,
        ApplicationCapabilityMagnitudeDimension, ApplicationCapabilityPropagationComposition,
        ApplicationCapabilityRelationBinding, ApplicationCapabilityRelationDimension,
        ApplicationCapabilitySeparationOfDutyRule, ApplicationCapabilityTargetDefinition,
        ApplicationCapabilityValidityDefinition, ApplicationCapabilityValidityTimeline,
        ApplicationCapabilityValueBinding, ApplicationCapabilityWorkflowDefinition,
    },
    application_schema::{
        ApplicationAuthorizationPathBuilder, ApplicationEncodedScalarValue,
        ApplicationSchemaDeclarationBuilder, StringApplicationValueBinding,
    },
};

use super::super::schema::{BoundedDimensionSchema, Part, PartIdentityField, Principal};
use super::advance::{
    WorkflowAdvanceCapability, WorkflowAdvanceContext, WorkflowAdvanceInput,
    WorkflowAdvanceOperation, WorkflowAdvanceProvenance, WorkflowApprovalCapability,
};
use super::declaration::{
    WorkflowAuthoringGrant, WorkflowDefinitionAuthoringCapability,
    WorkflowDefinitionAuthoringContext, WorkflowDefinitionAuthoringInput,
    WorkflowDefinitionAuthoringOperation, WorkflowDefinitionAuthoringProvenance,
    WorkflowGrantActionField, WorkflowGrantDelegationLimitField, WorkflowGrantGrantee,
    WorkflowGrantGrantor, WorkflowGrantNotAfterField, WorkflowGrantNotBeforeField,
    WorkflowGrantParent, WorkflowGrantPurposeField, WorkflowGrantRelated, WorkflowGrantResource,
    WorkflowGrantStatusField, WorkflowGrantWorkflowField, WorkflowInstanceStartCapability,
    WorkflowInstanceStartContext, WorkflowInstanceStartInput, WorkflowInstanceStartOperation,
    WorkflowInstanceStartProvenance, WorkflowPartOwner,
};

pub(super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<BoundedDimensionSchema>,
) -> ApplicationSchemaDeclarationBuilder<BoundedDimensionSchema> {
    schema
        .capability(contract())
        .capability(start_contract())
        .capability(advance_contract())
        .capability(approval_contract())
}

fn approval_contract() -> ApplicationCapabilityContract<
    BoundedDimensionSchema,
    WorkflowApprovalCapability,
    WorkflowAdvanceOperation,
    WorkflowAdvanceInput,
> {
    ApplicationCapabilityContractBuilder::new(
        WorkflowApprovalCapability::reference(),
        WorkflowAdvanceOperation::reference(),
        WorkflowAuthoringGrant::reference(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            WorkflowGrantActionField::reference(),
            encoded("approve-workflow-transition"),
        ),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(
            WorkflowGrantPurposeField::reference(),
            encoded("reviewed-geometry"),
        ),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                WorkflowGrantStatusField::reference(),
                encoded("active"),
            ),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(
                    WorkflowGrantWorkflowField::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(PartIdentityField::reference()),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(
                    WorkflowGrantNotBeforeField::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(
                    WorkflowGrantNotAfterField::reference(),
                ),
            ),
        ),
        WorkflowAdvanceContext::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(
            WorkflowGrantDelegationLimitField::reference(),
        ),
        WorkflowAdvanceProvenance::reference(),
    ))
    .composition(composition())
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}

fn advance_contract() -> ApplicationCapabilityContract<
    BoundedDimensionSchema,
    WorkflowAdvanceCapability,
    WorkflowAdvanceOperation,
    WorkflowAdvanceInput,
> {
    ApplicationCapabilityContractBuilder::new(
        WorkflowAdvanceCapability::reference(),
        WorkflowAdvanceOperation::reference(),
        WorkflowAuthoringGrant::reference(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            WorkflowGrantActionField::reference(),
            encoded("advance-workflow-instance"),
        ),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantResource::reference()),
        ApplicationCapabilityRelationDimension::bound(WorkflowGrantRelated::reference()),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(
            WorkflowGrantPurposeField::reference(),
            encoded("reviewed-geometry"),
        ),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                WorkflowGrantStatusField::reference(),
                encoded("active"),
            ),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(
                    WorkflowGrantWorkflowField::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(PartIdentityField::reference()),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(
                    WorkflowGrantNotBeforeField::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(
                    WorkflowGrantNotAfterField::reference(),
                ),
            ),
        ),
        WorkflowAdvanceContext::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(
            WorkflowGrantDelegationLimitField::reference(),
        ),
        WorkflowAdvanceProvenance::reference(),
    ))
    .composition(composition())
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}

fn start_contract() -> ApplicationCapabilityContract<
    BoundedDimensionSchema,
    WorkflowInstanceStartCapability,
    WorkflowInstanceStartOperation,
    WorkflowInstanceStartInput,
> {
    ApplicationCapabilityContractBuilder::new(
        WorkflowInstanceStartCapability::reference(),
        WorkflowInstanceStartOperation::reference(),
        WorkflowAuthoringGrant::reference(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            WorkflowGrantActionField::reference(),
            encoded("start-workflow-instance"),
        ),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(
            WorkflowGrantPurposeField::reference(),
            encoded("reviewed-geometry"),
        ),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                WorkflowGrantStatusField::reference(),
                encoded("active"),
            ),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(
                    WorkflowGrantWorkflowField::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(PartIdentityField::reference()),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(
                    WorkflowGrantNotBeforeField::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(
                    WorkflowGrantNotAfterField::reference(),
                ),
            ),
        ),
        WorkflowInstanceStartContext::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(
            WorkflowGrantDelegationLimitField::reference(),
        ),
        WorkflowInstanceStartProvenance::reference(),
    ))
    .composition(composition())
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}

fn contract() -> ApplicationCapabilityContract<
    BoundedDimensionSchema,
    WorkflowDefinitionAuthoringCapability,
    WorkflowDefinitionAuthoringOperation,
    WorkflowDefinitionAuthoringInput,
> {
    ApplicationCapabilityContractBuilder::new(
        WorkflowDefinitionAuthoringCapability::reference(),
        WorkflowDefinitionAuthoringOperation::reference(),
        WorkflowAuthoringGrant::reference(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            WorkflowGrantActionField::reference(),
            encoded("author-workflow-definition"),
        ),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantResource::reference()),
        ApplicationCapabilityRelationDimension::bound(WorkflowGrantRelated::reference()),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(
            WorkflowGrantPurposeField::reference(),
            encoded("reviewed-geometry"),
        ),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                WorkflowGrantStatusField::reference(),
                encoded("active"),
            ),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(
                    WorkflowGrantWorkflowField::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(PartIdentityField::reference()),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(
                    WorkflowGrantNotBeforeField::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(
                    WorkflowGrantNotAfterField::reference(),
                ),
            ),
        ),
        WorkflowDefinitionAuthoringContext::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(
            WorkflowGrantDelegationLimitField::reference(),
        ),
        WorkflowDefinitionAuthoringProvenance::reference(),
    ))
    .composition(composition())
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}

fn composition() -> ApplicationCapabilityComposition {
    let allow = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(WorkflowPartOwner::reference())
        .allow(Part::reference());
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
        ),
        ApplicationCapabilityPropagationComposition::new(
            ApplicationCapabilityDelegationRule::forbidden(),
            ApplicationCapabilityDisclosureRule::not_applicable(),
        ),
    )
}

fn encoded(value: &str) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value.to_owned())
        .expect("workflow capability fixture text must encode")
}
