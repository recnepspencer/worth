pub use bank_domain::schema::{
    ApproveEstateEmergencyAccessOperation, BankSchema, CompleteEstateMandatoryReviewOperation,
    DelegateEstateCapabilityOperation, DisburseEstateOperation, FreezeEstateAccountOperation,
    NotifyDeathEstateOperation, OpenEstateCaseOperation, RecognizeEstateExecutorOperation,
    ReleaseEstateOperation, RequestEstateEmergencyAccessOperation,
    RetransmitDeathNoticeEstateOperation, RevokeEstateCapabilityOperation,
    RevokeEstateEmergencyAccessOperation, ViewEstateAdministrationCapability,
    ViewRestrictedEstateOperation,
};
pub use worth_query_host::facade::declaration::application_schema::{
    ApplicationOperationRef, ApplicationSchemaMember,
};
pub use worth_query_host::facade::domain::{
    PublishedAftermathPosture, WorthQueryInstallationAdmissionProfile,
    WorthQueryInstallationGeneration, WorthQueryInstallationRuntimeIdentity,
    WorthQueryInstalledApplicationSchema, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};

pub fn installed_bank() -> WorthQueryInstalledApplicationSchema<BankSchema> {
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "WORTH.bank",
        1,
        0,
    ))
    .application_schema(BankSchema::declaration().unwrap())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support-v1", "config-v1")
        .admit(package)
        .unwrap();
    let index = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap();
    index
        .bind_application_schema(BankSchema::declaration().unwrap())
        .unwrap()
}

pub fn assert_installed_posture<Operation, Input>(
    bank: &WorthQueryInstalledApplicationSchema<BankSchema>,
    operation: ApplicationOperationRef<BankSchema, Operation, Input>,
    expected: Option<PublishedAftermathPosture>,
    label: &str,
) where
    Operation: worth_query_host::facade::declaration::application_schema::ApplicationOperationMarkerIdentity<
            BankSchema,
        > + 'static,
    Input: 'static,
    Operation::InputBinding:
        worth_query_host::facade::declaration::application_schema::ApplicationStructuredValueBinding<
            Value = Input,
        >,
{
    let installed = bank
        .installed_operation(operation)
        .unwrap_or_else(|denial| panic!("{label} must install: {denial:?}"));
    let posture = installed
        .contracts()
        .aftermath()
        .map(|contract| contract.published_posture());
    assert_eq!(posture, expected, "{label} aftermath posture");
}
