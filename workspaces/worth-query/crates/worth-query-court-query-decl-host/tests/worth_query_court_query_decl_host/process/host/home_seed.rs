//! Producer seeds refuse at preparation until producer custody can close a Home.
use worth_foundational::facade::DurableStateFamily;
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_installation::{
        ApplicationBuildIdentity, WorthQueryApplicationPreparationDenial,
        WorthQueryApplicationProgramRoster,
    },
    product::WorthQueryDurableStateAdmissionDenial,
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeOpenRefused,
    WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_host::facade::host::{application_home, checkpoint_capture};

#[test]
fn directory_producer_seed_refuses_before_initialization_or_owner_effect() {
    use worth_query_testkit_document_retention::facade::document_retention_model::{
        host::host_limits, programs::RetentionProgramP0, retention_entry::assessment_counts,
        schema::DocumentRetentionSchema,
    };
    let directory = application_home::location();
    assert!(!directory.exists());
    let declaration = DocumentRetentionSchema::declaration().unwrap();
    let contacts = assessment_counts(6);
    let request = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(directory.clone()),
        ApplicationProgramAuthoring::<DocumentRetentionSchema, RetentionProgramP0>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        ((),),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        |_, _, _| panic!("an unsupported producer seed cannot initialize"),
    );
    let lease = checkpoint_capture::lease(128 << 20);
    let refused = with_bootstrap_advancement_from_request(
        worth_execution::ExecutionRequest::leased(&lease),
        |phase| Box::new(request.open_in_advancement(&phase)),
    )
    .unwrap()
    .err()
    .expect("declared producers need producer custody before a Home can close");
    assert!(
        matches!(
            refused,
            WorthQueryApplicationHomeOpenRefused::Preparation(
                WorthQueryApplicationPreparationDenial::UnsupportedSeed(
                    WorthQueryDurableStateAdmissionDenial::Unsupported {
                        family: DurableStateFamily::ProducerCleanupCustody,
                        supported_from_phase: 7,
                    }
                )
            )
        ),
        "exact producer seed refusal: {refused:?}"
    );
    assert_eq!(assessment_counts(6), contacts);
    assert!(
        !directory.exists(),
        "no descriptor, journal or bootstrap may be created"
    );
}
