use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityActorComposition, ApplicationCapabilityAllowRule,
        ApplicationCapabilityCardinalityDimension, ApplicationCapabilityComposition,
        ApplicationCapabilityConflictRule, ApplicationCapabilityConstraintDefinition,
        ApplicationCapabilityContract, ApplicationCapabilityContractBuilder,
        ApplicationCapabilityCurrentnessDefinition, ApplicationCapabilityDecisionComposition,
        ApplicationCapabilityDelegationDefinition, ApplicationCapabilityDenyRule,
        ApplicationCapabilityDistinctActorRule, ApplicationCapabilityElevationRule,
        ApplicationCapabilityFieldBinding, ApplicationCapabilityFieldDimension,
        ApplicationCapabilityGraphClause, ApplicationCapabilityGraphRule,
        ApplicationCapabilityMagnitudeDimension, ApplicationCapabilityPathContextAnchor,
        ApplicationCapabilityRelationBinding, ApplicationCapabilityRelationDimension,
        ApplicationCapabilitySeparationOfDutyRule, ApplicationCapabilityTargetDefinition,
        ApplicationCapabilityValidityDefinition, ApplicationCapabilityValidityTimeline,
        ApplicationCapabilityValueBinding, ApplicationCapabilityWorkflowDefinition,
    },
    application_schema::{
        ApplicationAuthorizationPathBuilder, ApplicationEncodedScalarValue,
        ApplicationSchemaDeclarationBuilder,
    },
};

use super::super::{
    Account, AccountIdentity, AccountLabel, AccountOwner, AccountStatus, IdentityExecutionSchema,
    Principal,
};
use super::declaration::*;

#[path = "propagation.rs"]
mod propagation;
use propagation::capability_propagation;

pub(in super::super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema>,
) -> ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema> {
    let schema = install_grant_facts(schema);
    let schema = install_grant_relations(schema);
    install_capability_operations(schema)
}

fn install_grant_facts(
    schema: ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema>,
) -> ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema> {
    schema
}

fn install_grant_relations(
    schema: ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema>,
) -> ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema> {
    schema
}

fn install_capability_operations(
    schema: ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema>,
) -> ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema> {
    schema
        .capability_context(CapabilityRequestContext::reference())
        .capability_context_entity_slot(CapabilityRequestActorSlot::reference())
        .capability_context_entity_slot(CapabilityPriorActorSlot::reference())
        .capability_provenance(CapabilityProvenance::reference())
        .operation(
            CapabilityTouchOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(CapabilityTouchOperation::reference(), 32)
        .operation_read_field(
            CapabilityTouchOperation::reference(),
            AccountIdentity::at("AccountIdentity"),
        )
        .operation_read_field(
            CapabilityTouchOperation::reference(),
            AccountLabel::at("AccountLabel"),
        )
        .operation_write(
            CapabilityTouchOperation::reference(),
            AccountLabel::at("AccountLabel"),
        )
        .operation(
            ComposedCapabilityTouchOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(ComposedCapabilityTouchOperation::reference(), 32)
        .operation_read_field(
            ComposedCapabilityTouchOperation::reference(),
            AccountIdentity::at("AccountIdentity"),
        )
        .operation_read_field(
            ComposedCapabilityTouchOperation::reference(),
            CapabilityActionRecordIdentity::at("CapabilityActionRecordIdentity"),
        )
        .operation_read_field(
            ComposedCapabilityTouchOperation::reference(),
            AccountLabel::at("AccountLabel"),
        )
        .operation_write(
            ComposedCapabilityTouchOperation::reference(),
            AccountLabel::at("AccountLabel"),
        )
        .capability(capability_contract())
        .capability(composed_capability_contract())
}

fn capability_contract() -> ApplicationCapabilityContract<
    IdentityExecutionSchema,
    TouchAccountCapability,
    CapabilityTouchOperation,
    CapabilityTouchInput,
> {
    ApplicationCapabilityContractBuilder::new(
        TouchAccountCapability::reference(),
        CapabilityTouchOperation::reference(),
        CapabilityGrant::reference(),
    )
    .target(capability_target())
    .constraints(capability_constraints())
    .delegation(capability_delegation())
    .composition(capability_composition())
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}

fn composed_capability_contract() -> ApplicationCapabilityContract<
    IdentityExecutionSchema,
    ComposedTouchAccountCapability,
    ComposedCapabilityTouchOperation,
    CapabilityTouchInput,
> {
    ApplicationCapabilityContractBuilder::new(
        ComposedTouchAccountCapability::reference(),
        ComposedCapabilityTouchOperation::reference(),
        CapabilityGrant::reference(),
    )
    .target(capability_target())
    .constraints(capability_constraints())
    .delegation(capability_delegation())
    .composition(composed_capability_composition())
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}

fn capability_target() -> ApplicationCapabilityTargetDefinition {
    ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            CapabilityActionField::at("CapabilityActionField"),
            ApplicationEncodedScalarValue::<CapabilityActionBinding>::try_new(
                CapabilityAction::Touch,
            )
            .unwrap(),
        )
        .unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(CapabilityResource::reference()),
        ApplicationCapabilityRelationDimension::bound(CapabilityRelated::reference()),
        ApplicationCapabilityFieldDimension::bound(CapabilityDisclosureField::at(
            "CapabilityDisclosureField",
        ))
        .unwrap(),
        ApplicationCapabilityValueBinding::new(
            CapabilityPurposeField::at("CapabilityPurposeField"),
            ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(
                CapabilityPurpose::AccountMaintenance,
            )
            .unwrap(),
        )
        .unwrap(),
    )
}

fn capability_constraints() -> ApplicationCapabilityConstraintDefinition {
    ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::bound(CapabilityAmountField::at(
            "CapabilityAmountField",
        ))
        .expect("record field contract"),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                CapabilityStatusField::at("CapabilityStatusField"),
                ApplicationEncodedScalarValue::<CapabilityStatusBinding>::try_new(
                    CapabilityStatus::Active,
                )
                .unwrap(),
            )
            .unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(CapabilityWorkflowField::at(
                    "CapabilityWorkflowField",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(AccountStatus::at(
                    "AccountStatus",
                ))
                .unwrap(),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(CapabilityNotBeforeField::at(
                    "CapabilityNotBeforeField",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(CapabilityNotAfterField::at(
                    "CapabilityNotAfterField",
                ))
                .unwrap(),
            ),
        ),
        CapabilityRequestContext::reference(),
    )
}

fn capability_delegation() -> ApplicationCapabilityDelegationDefinition {
    ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(CapabilityParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(CapabilityGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(CapabilityGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(CapabilityDelegationLimitField::at(
            "CapabilityDelegationLimitField",
        ))
        .unwrap(),
        CapabilityProvenance::reference(),
    )
}

fn capability_composition() -> ApplicationCapabilityComposition {
    let grantor = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(CapabilityGrantor::reference())
        .forward(CapabilityResource::reference())
        .allow(Account::reference());
    let custodian = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(CapabilityCustodian::reference())
        .forward(CapabilityResource::reference())
        .allow(Account::reference());
    ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(ApplicationCapabilityGraphRule::any([
                ApplicationCapabilityGraphClause::new(grantor),
                ApplicationCapabilityGraphClause::new(custodian),
            ])),
            ApplicationCapabilityDenyRule::not_applicable(),
            ApplicationCapabilityConflictRule::not_applicable(),
        ),
        ApplicationCapabilityActorComposition::new(
            ApplicationCapabilitySeparationOfDutyRule::not_applicable(),
            ApplicationCapabilityDistinctActorRule::not_applicable(), worth_query_declaration::facade::application_capability::ApplicationCapabilityRecordedActorRule::not_applicable()),
        capability_propagation(),
    )
}

fn composed_capability_composition() -> ApplicationCapabilityComposition {
    let assignment = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(AccountOwner::reference())
        .allow(Account::reference());
    let explicit_deny = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(CapabilityExplicitDeny::reference())
        .deny(Account::reference());
    let conflict = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(CapabilityConflictingBeneficiary::reference())
        .deny(Account::reference());
    let request_actor = ApplicationCapabilityGraphClause::new(
        ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
            .forward(CapabilityRequestActor::reference())
            .forward(CapabilityActionResource::reference())
            .deny(Account::reference()),
    )
    .anchored([ApplicationCapabilityPathContextAnchor::after_forward(
        CapabilityRequestActor::reference(),
        CapabilityRequestActorSlot::reference(),
    )]);
    let prior_actor = ApplicationCapabilityGraphClause::new(
        ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
            .forward(CapabilityPriorActor::reference())
            .forward(CapabilityActionResource::reference())
            .deny(Account::reference()),
    )
    .anchored([ApplicationCapabilityPathContextAnchor::after_forward(
        CapabilityPriorActor::reference(),
        CapabilityPriorActorSlot::reference(),
    )]);
    let graph =
        |path| ApplicationCapabilityGraphRule::any([ApplicationCapabilityGraphClause::new(path)]);
    let anchored_graph = |clause| ApplicationCapabilityGraphRule::any([clause]);
    ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(graph(assignment)),
            ApplicationCapabilityDenyRule::when(graph(explicit_deny)),
            ApplicationCapabilityConflictRule::when(graph(conflict)),
        ),
        ApplicationCapabilityActorComposition::new(
            ApplicationCapabilitySeparationOfDutyRule::when(anchored_graph(request_actor)),
            ApplicationCapabilityDistinctActorRule::when(anchored_graph(prior_actor)), worth_query_declaration::facade::application_capability::ApplicationCapabilityRecordedActorRule::not_applicable()),
        capability_propagation(),
    )
}
