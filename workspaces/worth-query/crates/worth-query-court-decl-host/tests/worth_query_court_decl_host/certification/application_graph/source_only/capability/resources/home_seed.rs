//! Resource pools require authored owned-async definitions before they can serve.
use std::sync::Arc;
use worth_foundational::facade::DurableStateFamily;
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_installation::{
        ApplicationBuildIdentity, WorthQueryApplicationProgramRoster,
        WorthQueryInMemoryApplicationDenial,
    },
    product::WorthQueryDurableStateAdmissionDenial,
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeOpenRefused,
    WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_certification_operation::facade::certification::source_only::capability::{
    handler::Contacts, program::Program, schema::CapabilitySchema,
};

#[test]
fn directory_resource_seed_refuses_before_clock_provider_or_owner_effect() {
    let directory = std::env::temp_dir().join(format!(
        "worth-resource-seed-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    ));
    assert!(!directory.exists());
    let contacts = Arc::new(Contacts::default());
    let declaration = CapabilitySchema::declaration().unwrap();
    let request = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(directory.clone()),
        ApplicationProgramAuthoring::<CapabilitySchema, Program>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        (contacts.clone(),),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        worth_query_testkit_document_retention::facade::document_retention_model::host::host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        |_, _, _| panic!("an unsupported resource seed cannot initialize"),
    );
    let budget = worth_execution::SerialMemoryBudget::new(Some(128 << 20));
    let cancellation = worth_execution::CancellationSource::new();
    let host = worth_execution::SerialRequest::from_memory(budget, cancellation.token(), None);
    let refused = with_bootstrap_advancement_from_request(
        worth_execution::ExecutionRequest::serial(&host),
        |phase| Box::new(request.open_in_advancement(&phase)),
    )
    .unwrap()
    .err()
    .expect("resource pools require authored owned-async recovery");
    assert!(
        matches!(
            refused,
            WorthQueryApplicationHomeOpenRefused::Preparation(
                WorthQueryInMemoryApplicationDenial::UnsupportedSeed(
                    WorthQueryDurableStateAdmissionDenial::Unsupported {
                        family: DurableStateFamily::BridgeOwnedAsyncDefinitions,
                        supported_from_phase: 5,
                    }
                )
            )
        ),
        "exact resource seed refusal: {refused:?}"
    );
    assert_eq!(contacts.counts(), (0, 0));
    assert_eq!(contacts.resource.counts(), (0, 0, 0));
    assert_eq!(contacts.resource_clock.observations(), 0);
    assert!(
        !directory.exists(),
        "no descriptor, journal or bootstrap may be created"
    );
}
