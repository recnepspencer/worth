use super::*;

pub(super) fn install(
    schema: worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder<
        IdentityExecutionSchema,
    >,
) -> worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder<
    IdentityExecutionSchema,
> {
    schema
        .operation(
            TouchAccountOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(TouchAccountOperation::reference(), 2)
        .operation_projection_work_budget(TouchAccountOperation::reference(), 32)
        .operation_requires_ability(TouchAccountOperation::reference(), ViewAccount::reference())
        .operation_write(
            TouchAccountOperation::reference(),
            AccountStatus::reference(),
        )
        .operation_write(
            TouchAccountOperation::reference(),
            AccountLabel::reference(),
        )
        .operation_delete(TouchAccountOperation::reference(), Account::reference())
        .operation_emit(
            TouchAccountOperation::reference(),
            AccountActivityEffect::reference(),
        )
        .operation_emit(
            TouchAccountOperation::reference(),
            LiveActivityEffect::reference(),
        )
        .operation_read_field(
            TouchAccountOperation::reference(),
            AccountStatus::reference(),
        )
        .operation_read_field(
            TouchAccountOperation::reference(),
            AccountLabel::reference(),
        )
        .operation_expected_fact(
            TouchAccountOperation::reference(),
            AccountStatus::reference(),
        )
        .operation(
            PatchAccountDraftOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(PatchAccountDraftOperation::reference(), 2)
        .operation_projection_work_budget(PatchAccountDraftOperation::reference(), 16)
        .operation_requires_ability(
            PatchAccountDraftOperation::reference(),
            ViewAccount::reference(),
        )
        .operation_write(
            PatchAccountDraftOperation::reference(),
            AccountNote::reference(),
        )
        .operation_write(
            PatchAccountDraftOperation::reference(),
            AccountScore::reference(),
        )
        .operation_read_field(
            PatchAccountDraftOperation::reference(),
            AccountNote::reference(),
        )
        .operation_read_field(
            PatchAccountDraftOperation::reference(),
            AccountScore::reference(),
        )
        .operation(
            WrongFieldRetentionOperation::reference()
                .definition()
                .no_external_effect()
                .aftermath(schema_types::wrong_field_retention::aftermath())
                .finish(),
        )
        .operation_decision_fact_budget(WrongFieldRetentionOperation::reference(), 2)
        .operation_projection_work_budget(WrongFieldRetentionOperation::reference(), 16)
        .operation_requires_ability(
            WrongFieldRetentionOperation::reference(),
            ViewAccount::reference(),
        )
        .operation_write(
            WrongFieldRetentionOperation::reference(),
            AccountLabel::reference(),
        )
        .operation_read_field(
            WrongFieldRetentionOperation::reference(),
            AccountStatus::reference(),
        )
        .operation_read_field(
            WrongFieldRetentionOperation::reference(),
            AccountLabel::reference(),
        )
        .operation(
            ExactStatusRetentionOperation::reference()
                .definition()
                .external_effect(
                    RetainedStatusEffect::reference(),
                    worth_query_installation::facade::WorthQueryExternalEffectCorrelationFamily::new(
                        "test-status-retention-rail",
                    )
                    .unwrap(),
                )
                .aftermath(schema_types::exact_preimage_retention::status_with_external_owner())
                .finish(),
        )
        .operation_decision_fact_budget(ExactStatusRetentionOperation::reference(), 2)
        .operation_projection_work_budget(ExactStatusRetentionOperation::reference(), 32)
        .operation_requires_ability(
            ExactStatusRetentionOperation::reference(),
            ViewAccount::reference(),
        )
        .operation_write(
            ExactStatusRetentionOperation::reference(),
            AccountStatus::reference(),
        )
        .operation_write(
            ExactStatusRetentionOperation::reference(),
            AccountLabel::reference(),
        )
        .operation_read_field(
            ExactStatusRetentionOperation::reference(),
            AccountStatus::reference(),
        )
        .operation_read_field(
            ExactStatusRetentionOperation::reference(),
            AccountLabel::reference(),
        )
        .operation_emit(
            ExactStatusRetentionOperation::reference(),
            RetainedStatusEffect::reference(),
        )
        .operation(
            MultiFieldRetentionOperation::reference()
                .definition()
                .no_external_effect()
                .aftermath(schema_types::exact_preimage_retention::two_field_inverse())
                .finish(),
        )
        .operation_decision_fact_budget(MultiFieldRetentionOperation::reference(), 2)
        .operation_projection_work_budget(MultiFieldRetentionOperation::reference(), 32)
        .operation_requires_ability(
            MultiFieldRetentionOperation::reference(),
            ViewAccount::reference(),
        )
        .operation_write(
            MultiFieldRetentionOperation::reference(),
            AccountStatus::reference(),
        )
        .operation_write(
            MultiFieldRetentionOperation::reference(),
            AccountLabel::reference(),
        )
        .operation_read_field(
            MultiFieldRetentionOperation::reference(),
            AccountStatus::reference(),
        )
        .operation_read_field(
            MultiFieldRetentionOperation::reference(),
            AccountLabel::reference(),
        )
        .operation(
            MultiTouchOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(MultiTouchOperation::reference(), 2)
        .operation_projection_work_budget(MultiTouchOperation::reference(), 32)
        .operation_requires_ability(MultiTouchOperation::reference(), ViewAccount::reference())
        .operation_requires_ability(MultiTouchOperation::reference(), EditAccount::reference())
        .operation_write(MultiTouchOperation::reference(), AccountStatus::reference())
        .operation_read_field(MultiTouchOperation::reference(), AccountStatus::reference())
        .operation(
            ChangeOwnershipOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(ChangeOwnershipOperation::reference(), 2)
        .operation_projection_work_budget(ChangeOwnershipOperation::reference(), 32)
        .operation_requires_ability(
            ChangeOwnershipOperation::reference(),
            ManageOwnership::reference(),
        )
        .operation_read_relation(
            ChangeOwnershipOperation::reference(),
            AccountOwner::reference(),
        )
        .operation_read_field(
            ChangeOwnershipOperation::reference(),
            AccountStatus::reference(),
        )
        .operation_link(
            ChangeOwnershipOperation::reference(),
            AccountOwner::reference(),
        )
        .operation_unlink(
            ChangeOwnershipOperation::reference(),
            AccountOwner::reference(),
        )
        .operation(
            MutationFreeEmitOperation::reference()
                .definition()
                .external_effect(
                    MutationFreeExternalEffect::reference(),
                    worth_query_installation::facade::WorthQueryExternalEffectCorrelationFamily::new(
                        "test-mutation-free-rail",
                    )
                    .unwrap(),
                )
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(MutationFreeEmitOperation::reference(), 1)
        .operation_projection_work_budget(MutationFreeEmitOperation::reference(), 8)
        .operation_emit(
            MutationFreeEmitOperation::reference(),
            MutationFreeExternalEffect::reference(),
        )
}
