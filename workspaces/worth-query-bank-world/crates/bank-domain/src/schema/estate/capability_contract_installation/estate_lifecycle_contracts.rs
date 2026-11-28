use super::*;

pub(super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let schema = install_contract!(
        schema,
        NotifyDeathEstateCapability,
        NotifyDeathEstateOperation,
        EstateCapabilityOperation::NotifyDeath,
        EstateCapabilityPurpose::EstateAdministration,
        no_relation,
        no_field,
        no_magnitude
    );
    let schema = install_contract!(
        schema,
        RetransmitDeathNoticeEstateCapability,
        RetransmitDeathNoticeEstateOperation,
        EstateCapabilityOperation::RetransmitDeathNotice,
        EstateCapabilityPurpose::EstateAdministration,
        no_relation,
        no_field,
        no_magnitude
    );
    let schema = install_contract!(
        schema,
        FreezeEstateAccountCapability,
        FreezeEstateAccountOperation,
        EstateCapabilityOperation::FreezeAccount,
        EstateCapabilityPurpose::EstateAdministration,
        account_relation,
        no_field,
        no_magnitude
    );
    let schema = install_contract!(
        schema,
        OpenEstateCaseCapability,
        OpenEstateCaseOperation,
        EstateCapabilityOperation::OpenEstateCase,
        EstateCapabilityPurpose::EstateAdministration,
        no_relation,
        no_field,
        no_magnitude
    );
    install_contract!(
        schema,
        RecognizeEstateExecutorCapability,
        RecognizeEstateExecutorOperation,
        EstateCapabilityOperation::RecognizeExecutor,
        EstateCapabilityPurpose::LegalCompliance,
        no_relation,
        no_field,
        no_magnitude
    )
}
