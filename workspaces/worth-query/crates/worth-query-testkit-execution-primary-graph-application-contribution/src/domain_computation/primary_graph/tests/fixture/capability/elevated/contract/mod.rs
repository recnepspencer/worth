use worth_query_declaration::facade::application_capability::ApplicationCapabilityAcceptedValues;
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
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDelegationDepth;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDelegationRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDenyRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDisclosureRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDistinctActorRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityFieldBinding;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityFieldDimension;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityGraphClause;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityGraphRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityMagnitudeDimension;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityPropagationComposition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRelationBinding;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRelationDimension;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityScopeGuard;
use worth_query_declaration::facade::application_capability::ApplicationCapabilitySeparationOfDutyRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityTargetDefinition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityDefinition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValueBinding;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityWorkflowDefinition;
use worth_query_declaration::facade::application_schema::ApplicationAuthorizationPathBuilder;
use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;
use worth_query_declaration::facade::application_schema::ApplicationOperationDefinition;
use worth_query_declaration::facade::application_schema::ApplicationOperationRef;
use worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder;

use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityAction;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityActionBinding;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityActionField;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityAmountField;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityConflictingBeneficiary;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityCustodian;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDelegationLimitField;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosure;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosureBinding;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosureField;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityGrant;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityGrantee;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityGrantor;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityNotAfterField;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityNotBeforeField;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityParent;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityProvenance;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityPurpose;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityPurposeBinding;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityPurposeField;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityRequestContext;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityResource;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityStatus;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityStatusBinding;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityStatusField;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityWorkflowField;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::ApproveCapabilityElevationOperation;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationSlot;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewSlot;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CompleteCapabilityReviewOperation;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::ElevatedCapabilityTouchInput;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::ElevatedCapabilityTouchOperation;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::ElevatedTouchAccountCapability;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::RequestCapabilityElevationOperation;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::RevokeCapabilityElevationOperation;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use crate::domain_computation::primary_graph::tests::fixture::AccountLabel;
use crate::domain_computation::primary_graph::tests::fixture::AccountStatus;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::Principal;

pub use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::propagation::command_propagation;
pub use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::propagation::propagation;

pub fn install(
    schema: ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema>,
) -> ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema> {
    let schema = schema
        .capability_context_entity_slot(CapabilityElevationSlot::reference())
        .capability_context_entity_slot(CapabilityReviewSlot::reference())
        .operation(without_external_effect_or_aftermath(
            ElevatedCapabilityTouchOperation::reference(),
        ))
        .operation(without_external_effect_or_aftermath(
            RequestCapabilityElevationOperation::reference(),
        ))
        .operation(without_external_effect_or_aftermath(
            ApproveCapabilityElevationOperation::reference(),
        ))
        .operation(without_external_effect_or_aftermath(
            RevokeCapabilityElevationOperation::reference(),
        ))
        .operation(without_external_effect_or_aftermath(
            CompleteCapabilityReviewOperation::reference(),
        ))
        .operation_projection_work_budget(ElevatedCapabilityTouchOperation::reference(), 32)
        .operation_read_field(
            ElevatedCapabilityTouchOperation::reference(),
            AccountIdentity::at("AccountIdentity"),
        )
        .operation_read_field(
            ElevatedCapabilityTouchOperation::reference(),
            AccountLabel::at("AccountLabel"),
        )
        .operation_write(
            ElevatedCapabilityTouchOperation::reference(),
            AccountLabel::at("AccountLabel"),
        )
        .capability(elevated_contract());
    let schema = crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::request::install(schema);
    let schema = crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::approval::install(schema);
    let schema = crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::close::install(schema);
    crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::review::install(schema)
}

pub fn target() -> ApplicationCapabilityTargetDefinition {
    ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            CapabilityActionField::at("CapabilityActionField"),
            ApplicationEncodedScalarValue::<CapabilityActionBinding>::try_new(
                CapabilityAction::Touch,
            )
            .expect("fixture capability action must encode"),
        )
        .expect("the native capability field is declared"),
        ApplicationCapabilityRelationBinding::from_reference(CapabilityResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::bound(CapabilityDisclosureField::at(
            "CapabilityDisclosureField",
        ))
        .expect("the native capability field is declared"),
        ApplicationCapabilityValueBinding::new(
            CapabilityPurposeField::at("CapabilityPurposeField"),
            ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(
                CapabilityPurpose::AccountMaintenance,
            )
            .expect("fixture capability purpose must encode"),
        )
        .expect("the native capability field is declared"),
    )
}

pub fn command_target(action: CapabilityAction) -> ApplicationCapabilityTargetDefinition {
    ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            CapabilityActionField::at("CapabilityActionField"),
            ApplicationEncodedScalarValue::<CapabilityActionBinding>::try_new(action)
                .expect("fixture capability action must encode"),
        )
        .expect("the native capability field is declared"),
        ApplicationCapabilityRelationBinding::from_reference(CapabilityResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(
            CapabilityPurposeField::at("CapabilityPurposeField"),
            ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(
                CapabilityPurpose::AccountMaintenance,
            )
            .expect("fixture capability purpose must encode"),
        )
        .expect("the native capability field is declared"),
    )
}

pub fn constraints() -> ApplicationCapabilityConstraintDefinition {
    ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::bound(CapabilityAmountField::at(
            "CapabilityAmountField",
        ))
        .expect("the native capability field is declared"),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                CapabilityStatusField::at("CapabilityStatusField"),
                ApplicationEncodedScalarValue::<CapabilityStatusBinding>::try_new(
                    CapabilityStatus::Active,
                )
                .expect("fixture capability status must encode"),
            )
            .expect("the native capability field is declared"),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(CapabilityWorkflowField::at(
                    "CapabilityWorkflowField",
                ))
                .expect("the native capability field is declared"),
                ApplicationCapabilityFieldBinding::from_reference(AccountStatus::at(
                    "AccountStatus",
                ))
                .expect("the native capability field is declared"),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(CapabilityNotBeforeField::at(
                    "CapabilityNotBeforeField",
                ))
                .expect("the native capability field is declared"),
                ApplicationCapabilityFieldBinding::from_reference(CapabilityNotAfterField::at(
                    "CapabilityNotAfterField",
                ))
                .expect("the native capability field is declared"),
            ),
        ),
        CapabilityRequestContext::reference(),
    )
}

pub fn command_constraints() -> ApplicationCapabilityConstraintDefinition {
    ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                CapabilityStatusField::at("CapabilityStatusField"),
                ApplicationEncodedScalarValue::<CapabilityStatusBinding>::try_new(
                    CapabilityStatus::Active,
                )
                .expect("fixture capability status must encode"),
            )
            .expect("the native capability field is declared"),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(CapabilityWorkflowField::at(
                    "CapabilityWorkflowField",
                ))
                .expect("the native capability field is declared"),
                ApplicationCapabilityFieldBinding::from_reference(AccountStatus::at(
                    "AccountStatus",
                ))
                .expect("the native capability field is declared"),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(CapabilityNotBeforeField::at(
                    "CapabilityNotBeforeField",
                ))
                .expect("the native capability field is declared"),
                ApplicationCapabilityFieldBinding::from_reference(CapabilityNotAfterField::at(
                    "CapabilityNotAfterField",
                ))
                .expect("the native capability field is declared"),
            ),
        ),
        CapabilityRequestContext::reference(),
    )
}

pub fn delegation() -> ApplicationCapabilityDelegationDefinition {
    ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(CapabilityParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(CapabilityGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(CapabilityGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(CapabilityDelegationLimitField::at(
            "CapabilityDelegationLimitField",
        ))
        .expect("the native capability field is declared"),
        CapabilityProvenance::reference(),
    )
}

pub fn composition() -> ApplicationCapabilityComposition {
    composition_with_propagation(propagation())
}

pub fn command_composition() -> ApplicationCapabilityComposition {
    composition_with_propagation(command_propagation())
}

#[path = "approval/mod.rs"]
pub mod approval;
#[path = "close/mod.rs"]
pub mod close;
#[path = "elevated_composition/mod.rs"]
pub mod elevated_composition;
#[path = "elevation/mod.rs"]
pub mod elevation;
#[path = "propagation/mod.rs"]
pub mod propagation;
#[path = "request/mod.rs"]
pub mod request;
#[path = "review/mod.rs"]
pub mod review;
use elevated_composition::composition_with_propagation;
use elevated_composition::elevated_contract;
use elevated_composition::without_external_effect_or_aftermath;
