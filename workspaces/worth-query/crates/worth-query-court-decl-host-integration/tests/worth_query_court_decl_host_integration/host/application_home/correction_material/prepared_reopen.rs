//! A genuine native grant survives while its real Home releases every live port.
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationCorrectionDenial,
    primary_graph::{QueryCorrectionMaterialDenial, WorthQueryCorrectionDerivationDenial},
};
#[test]
fn owning_close_discharges_the_unperformed_home_correction_without_rederiving() {
    let observer = label::correction_source::inverse_observation::LabelInverseObservation::begin();
    let location = fixture::location();
    let lease = checkpoint_capture::lease(256 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(location.clone(), execution);
    let receipt = edit(&home);
    let coordinates = worth_query_host::facade::primary_graph::WorthQueryRecordedCorrectionSource::from_untrusted_parts(
        receipt.committed_product_publication().composite_commit().clone(), receipt.commit_reference().clone()).coordinates();
    drop(receipt);
    let (input, earlier) = label::correction_source::preparation::retain_home_label_input(
        home.runtime().runtime(),
        &coordinates,
        "prepared-label",
        &scope(),
    );
    assert_eq!(input.input().label, "primary");
    assert_eq!(observer.count(), 1);
    let (same_input, same_grant) = label::correction_source::preparation::retain_home_label_input(
        home.runtime().runtime(),
        &coordinates,
        "prepared-label",
        &scope(),
    );
    assert_eq!(same_input.input().label, input.input().label);
    assert_eq!(
        observer.count(),
        1,
        "same-incarnation recovery uses the recorded inverse input"
    );
    drop(same_grant);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("close with authentic preparation retained: {refused:?}"))
        .reclaim();
    let home = open(location, execution);
    let request = scope();
    let runtime = home.runtime();
    let principal = label::authentication::authenticate_external(
        runtime.runtime().installed_schema(),
        "alice",
        Duration::from_secs(120),
        &request,
    );
    let limits =
        WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::new(8).unwrap());
    let source = runtime
        .request(&principal, &request)
        .corrections()
        .select_coordinates(&coordinates, limits)
        .unwrap();
    let denied = runtime
        .request(&principal, &request)
        .corrections()
        .reverse(&source)
        .idempotency("prepared-label")
        .limits(limits)
        .prepare()
        .err()
        .expect("the acknowledged no-effect terminal cannot become first use");
    assert!(matches!(
        denied,
        WorthQueryApplicationCorrectionDenial::Derivation(
            WorthQueryCorrectionDerivationDenial::Material(
                QueryCorrectionMaterialDenial::PreparationDischarged
            )
        )
    ));
    drop(source);
    assert_eq!(input.input().label, "primary");
    assert_eq!(
        observer.count(),
        1,
        "discharged commands never rerun inverse derivation or the provider"
    );
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("discharged preparation close: {refused:?}"))
        .reclaim();
    drop(earlier);
}
