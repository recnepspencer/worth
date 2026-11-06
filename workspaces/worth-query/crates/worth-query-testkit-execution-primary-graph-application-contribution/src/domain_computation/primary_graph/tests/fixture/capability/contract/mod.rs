use worth_query_declaration::facade::application_capability::ApplicationCapabilityActorComposition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityAllowRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityCardinalityDimension;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityComposition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityConflictRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityConstraintDefinition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityContract;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityContractBuilder;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityCurrentnessDefinition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDecisionComposition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDelegationDefinition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDenyRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDistinctActorRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityElevationRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityFieldBinding;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityFieldDimension;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityGraphClause;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityGraphRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityMagnitudeDimension;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityPathContextAnchor;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRelationBinding;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRelationDimension;
use worth_query_declaration::facade::application_capability::ApplicationCapabilitySeparationOfDutyRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityTargetDefinition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityDefinition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValueBinding;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityWorkflowDefinition;
use worth_query_declaration::facade::application_schema::ApplicationAuthorizationPathBuilder;
use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;
use worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder;

use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::*;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use crate::domain_computation::primary_graph::tests::fixture::AccountLabel;
use crate::domain_computation::primary_graph::tests::fixture::AccountOwner;
use crate::domain_computation::primary_graph::tests::fixture::AccountStatus;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::Principal;

use crate::domain_computation::primary_graph::tests::fixture::capability::contract::propagation::capability_propagation;

pub fn install(
    schema: ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema>,
) -> ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema> {
    let schema = install_grant_facts(schema);
    let schema = install_grant_relations(schema);
    install_capability_operations(schema)
}

pub fn install_capability_operations(
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
            AccountIdentity::reference(),
        )
        .operation_read_field(
            CapabilityTouchOperation::reference(),
            AccountLabel::reference(),
        )
        .operation_write(
            CapabilityTouchOperation::reference(),
            AccountLabel::reference(),
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
            AccountIdentity::reference(),
        )
        .operation_read_field(
            ComposedCapabilityTouchOperation::reference(),
            CapabilityActionRecordIdentity::reference(),
        )
        .operation_read_field(
            ComposedCapabilityTouchOperation::reference(),
            AccountLabel::reference(),
        )
        .operation_write(
            ComposedCapabilityTouchOperation::reference(),
            AccountLabel::reference(),
        )
        .capability(capability_contract())
        .capability(composed_capability_contract())
}

pub fn capability_contract() -> ApplicationCapabilityContract<
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

pub fn composed_capability_contract() -> ApplicationCapabilityContract<
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

pub fn capability_target() -> ApplicationCapabilityTargetDefinition {
    ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            CapabilityActionField::reference(),
            ApplicationEncodedScalarValue::<CapabilityActionBinding>::try_new(
                CapabilityAction::Touch,
            )
            .unwrap(),
        ),
        ApplicationCapabilityRelationBinding::from_reference(CapabilityResource::reference()),
        ApplicationCapabilityRelationDimension::bound(CapabilityRelated::reference()),
        ApplicationCapabilityFieldDimension::bound(CapabilityDisclosureField::reference()),
        ApplicationCapabilityValueBinding::new(
            CapabilityPurposeField::reference(),
            ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(
                CapabilityPurpose::AccountMaintenance,
            )
            .unwrap(),
        ),
    )
}

pub fn capability_constraints() -> ApplicationCapabilityConstraintDefinition {
    ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::bound(CapabilityAmountField::reference()),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                CapabilityStatusField::reference(),
                ApplicationEncodedScalarValue::<CapabilityStatusBinding>::try_new(
                    CapabilityStatus::Active,
                )
                .unwrap(),
            ),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(
                    CapabilityWorkflowField::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(AccountStatus::reference()),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(
                    CapabilityNotBeforeField::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(
                    CapabilityNotAfterField::reference(),
                ),
            ),
        ),
        CapabilityRequestContext::reference(),
    )
}

pub fn capability_delegation() -> ApplicationCapabilityDelegationDefinition {
    ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(CapabilityParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(CapabilityGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(CapabilityGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(
            CapabilityDelegationLimitField::reference(),
        ),
        CapabilityProvenance::reference(),
    )
}

pub fn capability_composition() -> ApplicationCapabilityComposition {
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

pub fn composed_capability_composition() -> ApplicationCapabilityComposition {
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

#[path = "grant_installation/mod.rs"]
pub mod grant_installation;
#[path = "propagation/mod.rs"]
pub mod propagation;
use grant_installation::install_grant_facts;
use grant_installation::install_grant_relations;
