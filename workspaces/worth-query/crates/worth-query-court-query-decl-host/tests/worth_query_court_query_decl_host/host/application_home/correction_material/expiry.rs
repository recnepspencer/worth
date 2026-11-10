//! Owning expiry preserves authored state and keys while ending only optional uses.
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationCorrectionDenial,
    primary_graph::{
        QueryCorrectionMaterialDenial as Material, WorthQueryCorrectionDerivationDenial,
        WorthQueryCorrectionRetentionDenial as Retention, WorthQueryCorrectionSelectionDenial,
    },
};
fn authored(home: &Home) -> (String, u64) {
    let request = scope();
    let runtime = home.runtime();
    let principal = label::authentication::authenticate_external(
        runtime.runtime().installed_schema(),
        "alice",
        Duration::from_secs(120),
        &request,
    );
    let read = runtime
        .request(&principal, &request)
        .query(label::AccountSummaryParameters)
        .execute()
        .unwrap();
    assert_eq!(read.rows().len(), 1);
    (
        read.rows()[0].label.clone(),
        read.receipt().inspect().basis().version(),
    )
}
#[test]
fn owning_expiry_refuses_pins_and_preserves_authored_state_and_the_original_key_after_reopen() {
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
        "expiry-pin",
        &scope(),
    );
    assert_eq!(input.input().label, "primary");
    let baseline = authored(&home);
    assert_eq!(baseline.0, "changed");
    assert!(matches!(
        home.runtime()
            .expire_optional_correction_material(&execution, &coordinates),
        Err(Retention::Material(Material::MaterialPinned))
    ));
    assert_eq!(authored(&home), baseline);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("close discharges the actual preparation: {refused:?}"))
        .reclaim();
    let home = open(location.clone(), execution);
    assert_eq!(authored(&home), baseline);
    let request = scope();
    {
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
        runtime
            .expire_optional_correction_material(&execution, &coordinates)
            .unwrap();
        runtime
            .expire_optional_correction_material(&execution, &coordinates)
            .unwrap();
        let expired = runtime
            .request(&principal, &request)
            .corrections()
            .reverse(&source)
            .inspect(limits)
            .unwrap_err();
        assert!(matches!(
            expired,
            WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::Material(Material::RetainedMaterialExpired)
            )
        ));
        drop(source);
        let retry = runtime
            .request(&principal, &request)
            .advanced()
            .mutate(LabelEdit {
                status: "open".into(),
                label: "changed".into(),
            })
            .without_source()
            .idempotency(&"original-label".to_owned())
            .execute_in_program(
                runtime,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(
            matches!(retry, Outcome::RecoveredCommitted(_)),
            "expiry cannot turn the actual Native key into first use"
        );
        drop(retry);
    }
    assert_eq!(
        observer.count(),
        1,
        "expiry and retry never derive another inverse"
    );
    assert_eq!(authored(&home), baseline);
    assert_eq!(
        count(&home, &execution),
        0,
        "successful expiry acknowledges disposal before close"
    );
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("expired material close: {refused:?}"))
        .reclaim();
    let home = open(location, execution);
    assert_eq!(authored(&home), baseline);
    {
        let runtime = home.runtime();
        let principal = label::authentication::authenticate_external(
            runtime.runtime().installed_schema(),
            "alice",
            Duration::from_secs(120),
            &request,
        );
        let limits =
            WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::new(8).unwrap());
        let expired = runtime
            .request(&principal, &request)
            .corrections()
            .select_coordinates(&coordinates, limits)
            .err()
            .expect("cold expiry stays unavailable");
        assert!(matches!(
            expired,
            WorthQueryApplicationCorrectionDenial::Source(
                WorthQueryCorrectionSelectionDenial::Material(Material::RetainedMaterialExpired)
            )
        ));
        runtime
            .expire_optional_correction_material(&execution, &coordinates)
            .unwrap();
    }
    assert_eq!(
        count(&home, &execution),
        0,
        "the matching durable interpretation was released, never reset at open"
    );
    assert_eq!(observer.count(), 1);
    assert_eq!(input.input().label, "primary");
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("expired reopened close: {refused:?}"))
        .reclaim();
    drop(earlier);
}

#[test]
fn foreign_coordinates_refuse_expiry_without_releasing_either_interpretation() {
    let lease = checkpoint_capture::lease(256 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let own_location = fixture::location();
    let foreign_location = fixture::location();
    let own = open(own_location.clone(), execution);
    let foreign = open(foreign_location.clone(), execution);
    drop(edit(&own));
    let receipt = edit(&foreign);
    let coordinates = worth_query_host::facade::primary_graph::WorthQueryRecordedCorrectionSource::from_untrusted_parts(
        receipt.committed_product_publication().composite_commit().clone(), receipt.commit_reference().clone()).coordinates();
    drop(receipt);
    let own_basis = authored(&own);
    let foreign_basis = authored(&foreign);
    assert!(matches!(
        own.runtime()
            .expire_optional_correction_material(&execution, &coordinates),
        Err(Retention::Material(Material::ForeignHome))
    ));
    assert_eq!(authored(&own), own_basis);
    assert_eq!(authored(&foreign), foreign_basis);
    assert_eq!(count(&own, &execution), 1);
    assert_eq!(count(&foreign, &execution), 1);
    own.close(&execution)
        .unwrap_or_else(|refused| panic!("own close: {refused:?}"))
        .reclaim();
    foreign
        .close(&execution)
        .unwrap_or_else(|refused| panic!("foreign close: {refused:?}"))
        .reclaim();
    for location in [own_location, foreign_location] {
        let home = open(location, execution);
        assert_eq!(
            count(&home, &execution),
            1,
            "foreign DATA never authorizes a durable release"
        );
        home.close(&execution)
            .unwrap_or_else(|refused| panic!("reopened close: {refused:?}"))
            .reclaim();
    }
}
