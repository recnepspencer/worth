use worth_query_decl::facade::{
    application_capability::{
        ApplicationCapabilityCardinalityDimension, ApplicationCapabilityConstraintDefinition,
        ApplicationCapabilityContractBuilder, ApplicationCapabilityCurrentnessDefinition,
        ApplicationCapabilityDelegationActivationDefinition,
        ApplicationCapabilityDelegationDefinition, ApplicationCapabilityFieldBinding,
        ApplicationCapabilityFieldDimension, ApplicationCapabilityMagnitudeDimension,
        ApplicationCapabilityRelationBinding, ApplicationCapabilityRelationDimension,
        ApplicationCapabilityRevocationDefinition, ApplicationCapabilityTargetDefinition,
        ApplicationCapabilityValidityDefinition, ApplicationCapabilityValidityTimeline,
        ApplicationCapabilityValueBinding, ApplicationCapabilityWorkflowDefinition,
    },
    application_schema::{
        ApplicationOperationDefinition, ApplicationOperationRef,
        ApplicationSchemaDeclarationBuilder, OperationEmits,
        WorthQueryExternalEffectCorrelationFamily,
    },
};

use super::*;
use crate::{
    estate::{
        declared_aftermath_for, CapabilityGrantStatus, EstateAction, EstateCapabilityOperation,
        EstateCapabilityPurpose, ESTATE_DEATH_NOTICE_RAIL,
    },
    schema::BankSchema,
};

macro_rules! relation_dimension {
    (account_relation) => {
        ApplicationCapabilityRelationDimension::bound(CapabilityAccount::reference())
    };
    (no_relation) => {
        ApplicationCapabilityRelationDimension::not_applicable()
    };
}

macro_rules! field_dimension {
    (field) => {
        ApplicationCapabilityFieldDimension::bound(CapabilityDisclosureField::reference())
    };
    (no_field) => {
        ApplicationCapabilityFieldDimension::not_applicable()
    };
}

macro_rules! magnitude_dimension {
    (magnitude) => {
        ApplicationCapabilityMagnitudeDimension::bound(CapabilityAmountCeilingField::reference())
    };
    (no_magnitude) => {
        ApplicationCapabilityMagnitudeDimension::not_applicable()
    };
}

macro_rules! install_contract {
    (
        $schema:expr,
        $capability:ident,
        $operation:ident,
        $action:expr,
        $purpose:expr,
        $relation:ident,
        $field:ident,
        $magnitude:ident
    ) => {{
        let contract = ApplicationCapabilityContractBuilder::new(
            $capability::reference(),
            $operation::reference(),
            CapabilityGrant::reference(),
        )
        .target(target(
            $action,
            $purpose,
            relation_dimension!($relation),
            field_dimension!($field),
        ))
        .constraints(constraints(magnitude_dimension!($magnitude)))
        .delegation(delegation())
        .composition(super::capability_composition::composition(
            $action, $purpose,
        ))
        .elevation(super::capability_elevation::rule($action, $purpose))
        .build();
        $schema.capability(contract)
    }};
}

macro_rules! install_view_contract {
    ($schema:expr, $capability:ident, $purpose:expr) => {
        install_contract!(
            $schema,
            $capability,
            ViewRestrictedEstateOperation,
            EstateCapabilityOperation::ViewRestrictedEstate,
            $purpose,
            no_relation,
            field,
            no_magnitude
        )
    };
}

#[path = "capability_contract_installation/delegation_contracts.rs"]
mod delegation_contracts;
#[path = "capability_contract_installation/emergency_access_contracts.rs"]
mod emergency_access_contracts;
#[path = "capability_contract_installation/estate_lifecycle_contracts.rs"]
mod estate_lifecycle_contracts;
#[path = "capability_contract_installation/restricted_view_contracts.rs"]
mod restricted_view_contracts;
#[path = "capability_contract_installation/settlement_contracts.rs"]
mod settlement_contracts;

pub(super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let schema = install_capability_dimensions(install_operations(schema));
    let schema = estate_lifecycle_contracts::install(schema);
    let schema = delegation_contracts::install(schema);
    let schema = emergency_access_contracts::install(schema);
    let schema = settlement_contracts::install(schema);
    restricted_view_contracts::install(schema)
}

fn install_capability_dimensions(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .capability_context(EstateActionContext::reference())
        .capability_context_entity_slot(EstateLegalAuthoritySlot::reference())
        .capability_context_entity_slot(EstateEmergencyAccessSlot::reference())
        .capability_context_entity_slot(EstateMandatoryReviewSlot::reference())
        .capability_provenance(EstateGrantChainProvenance::reference())
}

fn install_operations(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .operation(escaping_operation(
            NotifyDeathEstateOperation::reference(),
            EstateCapabilityOperation::NotifyDeath,
        ))
        .operation(escaping_operation(
            RetransmitDeathNoticeEstateOperation::reference(),
            EstateCapabilityOperation::RetransmitDeathNotice,
        ))
        .operation(contained_operation(
            FreezeEstateAccountOperation::reference(),
            EstateCapabilityOperation::FreezeAccount,
        ))
        .operation(contained_operation(
            OpenEstateCaseOperation::reference(),
            EstateCapabilityOperation::OpenEstateCase,
        ))
        .operation(contained_operation(
            RecognizeEstateExecutorOperation::reference(),
            EstateCapabilityOperation::RecognizeExecutor,
        ))
        .operation(contained_operation(
            DelegateEstateCapabilityOperation::reference(),
            EstateCapabilityOperation::DelegateCapability,
        ))
        .operation(contained_operation(
            RevokeEstateCapabilityOperation::reference(),
            EstateCapabilityOperation::RevokeCapability,
        ))
        .operation(contained_operation(
            RequestEstateEmergencyAccessOperation::reference(),
            EstateCapabilityOperation::RequestEmergencyAccess,
        ))
        .operation(contained_operation(
            ApproveEstateEmergencyAccessOperation::reference(),
            EstateCapabilityOperation::ApproveEmergencyAccess,
        ))
        .operation(contained_operation(
            RevokeEstateEmergencyAccessOperation::reference(),
            EstateCapabilityOperation::RevokeEmergencyAccess,
        ))
        .operation(contained_operation(
            CompleteEstateMandatoryReviewOperation::reference(),
            EstateCapabilityOperation::CompleteMandatoryReview,
        ))
        .operation(contained_operation(
            ReleaseEstateOperation::reference(),
            EstateCapabilityOperation::ReleaseEstate,
        ))
        .operation(contained_operation(
            DisburseEstateOperation::reference(),
            EstateCapabilityOperation::DisburseEstate,
        ))
        .operation(contained_operation(
            ViewRestrictedEstateOperation::reference(),
            EstateCapabilityOperation::ViewRestrictedEstate,
        ))
}

fn contained_operation<Operation>(
    operation: ApplicationOperationRef<BankSchema, Operation, EstateAction>,
    capability: EstateCapabilityOperation,
) -> ApplicationOperationDefinition<BankSchema, Operation, EstateAction> {
    match declared_aftermath_for(capability) {
        Some(contract) => operation
            .definition()
            .no_external_effect()
            .aftermath(contract)
            .finish(),
        None => operation
            .definition()
            .no_external_effect()
            .no_aftermath()
            .finish(),
    }
}

fn escaping_operation<Operation>(
    operation: ApplicationOperationRef<BankSchema, Operation, EstateAction>,
    capability: EstateCapabilityOperation,
) -> ApplicationOperationDefinition<BankSchema, Operation, EstateAction>
where
    EstateDeathNotificationEffect: OperationEmits<Operation>,
{
    let correlation_family =
        WorthQueryExternalEffectCorrelationFamily::new(ESTATE_DEATH_NOTICE_RAIL)
            .expect("the estate death-notice rail is an atomic identity");
    match declared_aftermath_for(capability) {
        Some(contract) => operation
            .definition()
            .external_effect(
                EstateDeathNotificationEffect::reference(),
                correlation_family,
            )
            .aftermath(contract)
            .finish(),
        None => operation
            .definition()
            .external_effect(
                EstateDeathNotificationEffect::reference(),
                correlation_family,
            )
            .no_aftermath()
            .finish(),
    }
}

fn target(
    action: EstateCapabilityOperation,
    purpose: EstateCapabilityPurpose,
    relation: ApplicationCapabilityRelationDimension,
    field: ApplicationCapabilityFieldDimension,
) -> ApplicationCapabilityTargetDefinition {
    ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            CapabilityOperationField::reference(),
            crate::schema::encoded_bank_value(action),
        ),
        ApplicationCapabilityRelationBinding::from_reference(CapabilityEstate::reference()),
        relation,
        field,
        ApplicationCapabilityValueBinding::new(
            CapabilityPurposeField::reference(),
            crate::schema::encoded_bank_value(purpose),
        ),
    )
}

fn constraints(
    magnitude: ApplicationCapabilityMagnitudeDimension,
) -> ApplicationCapabilityConstraintDefinition {
    ApplicationCapabilityConstraintDefinition::new(
        magnitude,
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                CapabilityGrantStatusField::reference(),
                crate::schema::encoded_bank_value(CapabilityGrantStatus::Active),
            ),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(
                    CapabilityWorkflowStageField::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(
                    EstateWorkflowStageField::reference(),
                ),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(
                    CapabilityValidFromField::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(
                    CapabilityValidThroughField::reference(),
                ),
            ),
        ),
        EstateActionContext::reference(),
    )
}

fn delegation() -> ApplicationCapabilityDelegationDefinition {
    ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(CapabilityParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(CapabilityGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(CapabilityGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(
            CapabilityDelegationLimitField::reference(),
        ),
        EstateGrantChainProvenance::reference(),
    )
    .with_activation(ApplicationCapabilityDelegationActivationDefinition::new(
        DelegateEstateCapabilityOperation::reference(),
        ApplicationCapabilityFieldBinding::from_reference(
            CapabilityGrantIdentityField::reference(),
        ),
    )
    .with_context_relations([
        ApplicationCapabilityRelationBinding::from_reference(
            CapabilityInstitution::reference(),
        ),
        ApplicationCapabilityRelationBinding::from_reference(CapabilityBranch::reference()),
    ]))
    .with_revocation(ApplicationCapabilityRevocationDefinition::new(
        RevokeEstateCapabilityOperation::reference(),
        ApplicationCapabilityFieldBinding::from_reference(
            CapabilityGrantIdentityField::reference(),
        ),
        ApplicationCapabilityValueBinding::new(
            CapabilityGrantStatusField::reference(),
            crate::schema::encoded_bank_value(CapabilityGrantStatus::Revoked),
        ),
    ))
}
