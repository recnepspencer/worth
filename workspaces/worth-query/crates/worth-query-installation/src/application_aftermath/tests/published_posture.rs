//! Published posture and axis-pair installation evidence.

use worth_query_declaration::facade::application_aftermath::{
    DeclaredApplicationAftermathContract, DeclaredReconciliationProcedure,
};

use super::super::{
    InstalledCorrectionAuthority, InstalledCorrectionMechanism, PublishedAftermathPosture,
    WorthQueryAftermathInstallationDenialKind,
};
use super::{binding, compensation, digest, recorded_inverse, AftermathInstall, Balance};

#[test]
fn runtime_alone_recorded_inverse_publishes_reversible() {
    let schema = digest(2);
    let installed = AftermathInstall::new(binding(digest(1), schema, 1), "freeze-account")
        .reads::<Balance>()
        .install(DeclaredApplicationAftermathContract::runtime_alone(
            recorded_inverse::<Balance>(),
        ))
        .expect("recorded inverse installs");
    assert!(installed.correction_disclosure().is_none());
    assert_eq!(
        installed.published_posture(),
        PublishedAftermathPosture::Reversible
    );
    assert_eq!(
        installed.authority(),
        InstalledCorrectionAuthority::RuntimeAlone
    );
    assert!(matches!(
        installed.mechanism(),
        Some(InstalledCorrectionMechanism::RecordedInverse(_))
    ));
    let Some(InstalledCorrectionMechanism::RecordedInverse(inverse)) = installed.mechanism() else {
        panic!("expected recorded inverse");
    };
    assert_eq!(
        inverse
            .lowering_correspondence()
            .resolved()
            .correspondence_identity(),
        &super::super::aftermath_owner_identity_digest(
            "worth-query.lowering-correspondence",
            "geometry-inverse",
            1,
            0,
        )
        .unwrap()
    );
    assert_eq!(
        inverse
            .lowering_correspondence()
            .resolved()
            .compatibility_generation(),
        1
    );
    assert_eq!(
        inverse
            .lowering_correspondence()
            .resolved()
            .graph_participation_identity(),
        &schema
    );
    assert_eq!(
        inverse.lowering_correspondence().correspondence_slot(),
        "geometry-inverse"
    );
    assert_eq!(installed.canonical().basis_preparation_count(), 1);
    assert_eq!(installed.canonical().digest_derivation_count(), 1);
    assert_eq!(installed.canonical().digest_text_materializations(), 0);
    let _ = installed.next_actions().posture();
}

#[test]
fn runtime_alone_compensation_publishes_compensatable() {
    let installed = AftermathInstall::new(binding(digest(3), digest(4), 1), "disburse")
        .install(DeclaredApplicationAftermathContract::runtime_alone(
            compensation(),
        ))
        .expect("compensation installs");
    assert_eq!(
        installed.published_posture(),
        PublishedAftermathPosture::Compensatable
    );
}

#[test]
fn external_owner_publishes_reconcilable() {
    let installed = AftermathInstall::new(binding(digest(5), digest(6), 1), "notify-death")
        .install(
            DeclaredApplicationAftermathContract::runtime_with_external_owner(
                compensation(),
                DeclaredReconciliationProcedure::new("confirm-death-notice").unwrap(),
            ),
        )
        .expect("reconcilable installs");
    assert_eq!(
        installed.published_posture(),
        PublishedAftermathPosture::Reconcilable
    );
    assert_eq!(
        installed
            .reconciliation()
            .expect("external-owner aftermath retains reconciliation")
            .procedure_slot(),
        "confirm-death-notice"
    );
}

#[test]
fn not_correctable_publishes_irreversible_without_undo_surface() {
    let installed = AftermathInstall::new(binding(digest(7), digest(8), 1), "release-estate")
        .install(DeclaredApplicationAftermathContract::not_correctable())
        .expect("irreversible installs");
    assert_eq!(
        installed.published_posture(),
        PublishedAftermathPosture::Irreversible
    );
    match installed.next_actions() {
        super::super::InstalledAftermathNextActionContract::Irreversible(actions) => {
            let _ = actions;
        }
        _ => panic!("irreversible must expose irreversible next actions only"),
    }
}

#[test]
fn authority_axis_drift_changes_installed_identity() {
    let package = digest(9);
    let schema = digest(10);
    let left = AftermathInstall::new(binding(package, schema, 1), "same-operation")
        .install(DeclaredApplicationAftermathContract::runtime_alone(
            compensation(),
        ))
        .unwrap();
    let right = AftermathInstall::new(binding(package, schema, 1), "same-operation")
        .install(
            DeclaredApplicationAftermathContract::runtime_with_external_owner(
                compensation(),
                DeclaredReconciliationProcedure::new("confirm").unwrap(),
            ),
        )
        .unwrap();
    assert_ne!(left.identity().bytes(), right.identity().bytes());
}

#[test]
fn mechanism_axis_drift_changes_installed_identity() {
    let package = digest(11);
    let schema = digest(12);
    let left = AftermathInstall::new(binding(package, schema, 1), "same-operation")
        .reads::<Balance>()
        .install(DeclaredApplicationAftermathContract::runtime_alone(
            recorded_inverse::<Balance>(),
        ))
        .unwrap();
    let right = AftermathInstall::new(binding(package, schema, 1), "same-operation")
        .reads::<Balance>()
        .install(DeclaredApplicationAftermathContract::runtime_alone(
            compensation(),
        ))
        .unwrap();
    assert_ne!(left.identity().bytes(), right.identity().bytes());
    assert_ne!(left.published_posture(), right.published_posture());
}

#[test]
fn external_effect_rejects_reversible_with_named_cause() {
    let schema = digest(14);
    let denial = AftermathInstall::new(binding(digest(13), schema, 1), "wire-instruction")
        .reads::<Balance>()
        .escaping()
        .install(DeclaredApplicationAftermathContract::runtime_alone(
            recorded_inverse::<Balance>(),
        ))
        .expect_err("escaping effect cannot install as reversible");
    assert_eq!(
        denial.kind(),
        WorthQueryAftermathInstallationDenialKind::ExternalEffectRejectsReversible
    );
    assert_eq!(denial.subject(), "escaped-rail");
}

#[test]
fn external_effect_allows_compensation_twin() {
    let installed = AftermathInstall::new(binding(digest(15), digest(16), 1), "wire-instruction")
        .escaping()
        .install(DeclaredApplicationAftermathContract::runtime_alone(
            compensation(),
        ))
        .expect("escaping effect may install as compensatable");
    assert_eq!(
        installed.published_posture(),
        PublishedAftermathPosture::Compensatable
    );
}
