//! A requested dependency resumes from the initial consumer's own custody.
use super::*;

#[test]
fn an_initial_consumer_alone_resumes_its_requested_held_middle() {
    let _guard = checkpoint_recovery_test_guard();
    let application = chain_application();
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut a = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    let mut b = request
        .demand(ChainDemand("anchor-b".into()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    settle!(a, request);
    settle!(b, request);
    take_all_decisions();
    let mut c = request
        .demand(ChainDemand("anchor-c".into()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    change_root_input!(request, application, 5, 0x9176_4400_u64);
    application.delay_next_output_readiness_delivery_for_test();
    assert!(matches!(
        c.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Pending
    ));
    let settled = (0..8).find_map(|_| match c.advance(&request).unwrap() {
        WorthQueryApplicationOutputDemandProgress::Settled(done) => Some(done),
        WorthQueryApplicationOutputDemandProgress::Pending
        | WorthQueryApplicationOutputDemandProgress::Progressed => None,
    });
    assert!(
        settled.is_some(),
        "the initial consumer resumes its requested middle without another caller; frozen entry={:?}, counters={:?}, published={:?}",
        application.entered_required_selection_is_frozen_for_test(),
        application.undelivered_publication_counts_for_test(),
        application.published_required_rows_for_test()
    );
    assert_eq!(
        take_all_decisions()
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        ["anchor-b", "anchor-c"]
    );
    assert_eq!(
        request
            .query(PlanarOutputRead {
                body_key: "anchor-a".into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        length(6)
    );
}
