//! Distinct failure mechanism: a refused large snapshot cannot obstruct disposal.
use super::*;
#[test]
fn explicit_close_disposes_original_after_home_ingress_publication_refuses() {
    let directory = std::env::temp_dir().join(format!(
        "worth-resource-limit-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let contacts = Arc::new(Contacts::default());
    let initialized = Arc::new(AtomicUsize::new(0));
    let budget = worth_execution::SerialMemoryBudget::new(Some(256 << 20));
    let cancellation = worth_execution::CancellationSource::new();
    let host = worth_execution::SerialRequest::from_memory(budget, cancellation.token(), None);
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request(directory, contacts.clone(), initialized).open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|cause| panic!("resource Home: {cause:?}"));
    let (mut handle, identity) = {
        let scope = request_scope();
        let actor = authenticate(&home, 1, &scope);
        let application = home.runtime().runtime();
        let handle = application
            .application_resources(&actor, &scope, application.current_world())
            .start::<Resource>(7)
            .unwrap_or_else(|stop| {
                let (_, cause) = stop.into_parts();
                panic!("resource Start: {cause:?}")
            });
        let identity = handle.obligation_identity().unwrap();
        let issued = contacts.resource.issued.lock().unwrap();
        for _ in 0..2 {
            issued[0].writer.write(&[255; 2048]).unwrap();
        }
        (handle, identity)
    };
    let refused = home
        .close(&execution)
        .err()
        .expect("accepted ingress exceeds the existing obligation-frame limit");
    assert!(
        matches!(
            refused.query_denial(),
            Some(WorthQueryHomeRetirementDenial::ManagedCleanup(
                QueryObligationDenial::MalformedFrame
            ))
        ),
        "{refused:?}"
    );
    assert_eq!(
        contacts.resource.counts(),
        (1, 0, 0),
        "snapshot refusal precedes provider or Native release"
    );
    let home = refused
        .resume()
        .unwrap_or_else(|cause| panic!("retained original: {cause:?}"));
    {
        let scope = request_scope();
        let actor = authenticate(&home, 1, &scope);
        let application = home.runtime().runtime();
        let resources =
            application.application_resources(&actor, &scope, application.current_world());
        let state = resources
            .retained_state_governed::<Resource, _, _, _>(
                &7,
                identity,
                metadata_access(&home, &actor, &scope),
            )
            .unwrap();
        assert!(!state.released);
        // Explicit Close intentionally disposes payloads. Its real release and
        // discharge must fit without repeating the refused ingress snapshot.
        resources.close(&mut handle).unwrap();
        assert_eq!(handle.obligation_identity(), Some(identity));
        let absent = resources
            .retained_state_governed::<Resource, _, _, _>(
                &7,
                identity,
                metadata_access(&home, &actor, &scope),
            )
            .unwrap_err();
        assert_eq!(absent.kind(), WorthQueryResourceStopKind::Unavailable);
    }
    assert_eq!(contacts.resource.counts(), (1, 0, 1));
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("original disposal close: {cause:?}"))
        .reclaim();
}
