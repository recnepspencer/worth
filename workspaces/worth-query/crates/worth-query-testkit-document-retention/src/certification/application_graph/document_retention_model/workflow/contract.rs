pub use worth_query_host::facade::declaration::{
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

pub use super::super::schema::{
    Document, DocumentIdentityField, DocumentRetentionSchema, Principal,
};
pub use super::advance::{
    WorkflowAdvanceCapability, WorkflowAdvanceContext, WorkflowAdvanceInput,
    WorkflowAdvanceOperation, WorkflowAdvanceProvenance, WorkflowApprovalCapability,
};
pub use super::declaration::{
    WorkflowAuthoringGrant, WorkflowDefinitionAuthoringCapability,
    WorkflowDefinitionAuthoringContext, WorkflowDefinitionAuthoringInput,
    WorkflowDefinitionAuthoringOperation, WorkflowDefinitionAuthoringProvenance,
    WorkflowDocumentOwner, WorkflowGrantActionField, WorkflowGrantDelegationLimitField,
    WorkflowGrantGrantee, WorkflowGrantGrantor, WorkflowGrantNotAfterField,
    WorkflowGrantNotBeforeField, WorkflowGrantParent, WorkflowGrantPurposeField,
    WorkflowGrantRelated, WorkflowGrantResource, WorkflowGrantStatusField,
    WorkflowGrantWorkflowField, WorkflowInstanceStartCapability, WorkflowInstanceStartContext,
    WorkflowInstanceStartInput, WorkflowInstanceStartOperation, WorkflowInstanceStartProvenance,
};

pub fn install(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    schema
        .capability(contract())
        .capability(start_contract())
        .capability(advance_contract())
        .capability(approval_contract())
}

pub fn approval_contract() -> ApplicationCapabilityContract<
    DocumentRetentionSchema,
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
            WorkflowGrantActionField::at("WorkflowGrantActionField"),
            encoded("approve-workflow-transition"),
        )
        .unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(
            WorkflowGrantPurposeField::at("WorkflowGrantPurposeField"),
            encoded("reviewed-document"),
        )
        .unwrap(),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                WorkflowGrantStatusField::at("WorkflowGrantStatusField"),
                encoded("active"),
            )
            .unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantWorkflowField::at(
                    "WorkflowGrantWorkflowField",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(DocumentIdentityField::at(
                    "DocumentIdentityField",
                ))
                .unwrap(),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantNotBeforeField::at(
                    "WorkflowGrantNotBeforeField",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantNotAfterField::at(
                    "WorkflowGrantNotAfterField",
                ))
                .unwrap(),
            ),
        ),
        WorkflowAdvanceContext::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantDelegationLimitField::at(
            "WorkflowGrantDelegationLimitField",
        ))
        .unwrap(),
        WorkflowAdvanceProvenance::reference(),
    ))
    .composition(recorded_proposer_composition())
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}

pub fn advance_contract() -> ApplicationCapabilityContract<
    DocumentRetentionSchema,
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
            WorkflowGrantActionField::at("WorkflowGrantActionField"),
            encoded("advance-workflow-instance"),
        )
        .unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantResource::reference()),
        ApplicationCapabilityRelationDimension::bound(WorkflowGrantRelated::reference()),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(
            WorkflowGrantPurposeField::at("WorkflowGrantPurposeField"),
            encoded("reviewed-document"),
        )
        .unwrap(),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                WorkflowGrantStatusField::at("WorkflowGrantStatusField"),
                encoded("active"),
            )
            .unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantWorkflowField::at(
                    "WorkflowGrantWorkflowField",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(DocumentIdentityField::at(
                    "DocumentIdentityField",
                ))
                .unwrap(),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantNotBeforeField::at(
                    "WorkflowGrantNotBeforeField",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantNotAfterField::at(
                    "WorkflowGrantNotAfterField",
                ))
                .unwrap(),
            ),
        ),
        WorkflowAdvanceContext::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantDelegationLimitField::at(
            "WorkflowGrantDelegationLimitField",
        ))
        .unwrap(),
        WorkflowAdvanceProvenance::reference(),
    ))
    .composition(composition())
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}

pub fn start_contract() -> ApplicationCapabilityContract<
    DocumentRetentionSchema,
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
            WorkflowGrantActionField::at("WorkflowGrantActionField"),
            encoded("start-workflow-instance"),
        )
        .unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(
            WorkflowGrantPurposeField::at("WorkflowGrantPurposeField"),
            encoded("reviewed-document"),
        )
        .unwrap(),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                WorkflowGrantStatusField::at("WorkflowGrantStatusField"),
                encoded("active"),
            )
            .unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantWorkflowField::at(
                    "WorkflowGrantWorkflowField",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(DocumentIdentityField::at(
                    "DocumentIdentityField",
                ))
                .unwrap(),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantNotBeforeField::at(
                    "WorkflowGrantNotBeforeField",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantNotAfterField::at(
                    "WorkflowGrantNotAfterField",
                ))
                .unwrap(),
            ),
        ),
        WorkflowInstanceStartContext::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantDelegationLimitField::at(
            "WorkflowGrantDelegationLimitField",
        ))
        .unwrap(),
        WorkflowInstanceStartProvenance::reference(),
    ))
    .composition(composition())
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}

pub fn contract() -> ApplicationCapabilityContract<
    DocumentRetentionSchema,
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
            WorkflowGrantActionField::at("WorkflowGrantActionField"),
            encoded("author-workflow-definition"),
        )
        .unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantResource::reference()),
        ApplicationCapabilityRelationDimension::bound(WorkflowGrantRelated::reference()),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(
            WorkflowGrantPurposeField::at("WorkflowGrantPurposeField"),
            encoded("reviewed-document"),
        )
        .unwrap(),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                WorkflowGrantStatusField::at("WorkflowGrantStatusField"),
                encoded("active"),
            )
            .unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantWorkflowField::at(
                    "WorkflowGrantWorkflowField",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(DocumentIdentityField::at(
                    "DocumentIdentityField",
                ))
                .unwrap(),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantNotBeforeField::at(
                    "WorkflowGrantNotBeforeField",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantNotAfterField::at(
                    "WorkflowGrantNotAfterField",
                ))
                .unwrap(),
            ),
        ),
        WorkflowDefinitionAuthoringContext::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantDelegationLimitField::at(
            "WorkflowGrantDelegationLimitField",
        ))
        .unwrap(),
        WorkflowDefinitionAuthoringProvenance::reference(),
    ))
    .composition(composition())
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}

pub fn composition() -> ApplicationCapabilityComposition {
    let allow = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(WorkflowDocumentOwner::reference())
        .allow(Document::reference());
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
            ApplicationCapabilityDistinctActorRule::not_applicable(), worth_query_decl::facade::application_capability::ApplicationCapabilityRecordedActorRule::not_applicable()),
        ApplicationCapabilityPropagationComposition::new(
            ApplicationCapabilityDelegationRule::forbidden(),
            ApplicationCapabilityDisclosureRule::not_applicable(),
        ),
    )
}

pub fn encoded(value: &str) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value.to_owned())
        .expect("workflow capability fixture text must encode")
}

pub fn recorded_proposer_composition() -> ApplicationCapabilityComposition {
    use worth_query_decl::facade::application_capability::{
        ApplicationCapabilityRecordedActorRequirement, ApplicationCapabilityRecordedActorRule,
        ApplicationRecordedActorComparison, ApplicationWorkflowRecordedActorSource,
        ApplicationWorkflowRecordedRole,
    };
    let ordinary = composition();
    ApplicationCapabilityComposition::new(
        ordinary.decision().clone(),
        ApplicationCapabilityActorComposition::new(
            ApplicationCapabilitySeparationOfDutyRule::not_applicable(),
            ApplicationCapabilityDistinctActorRule::not_applicable(),
            ApplicationCapabilityRecordedActorRule::required(
                ApplicationCapabilityRecordedActorRequirement::new(
                    ApplicationWorkflowRecordedActorSource::BoundProposal,
                    ApplicationWorkflowRecordedRole::Proposer,
                    ApplicationRecordedActorComparison::DifferentPrincipal,
                ),
            ),
        ),
        ordinary.propagation().clone(),
    )
}
