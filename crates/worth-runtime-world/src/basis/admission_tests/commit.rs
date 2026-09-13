use super::{admit_current, component_fixture};

use crate::history::{
    CompositeCommitConstructionDenial, CompositeComponentChangePosture, CompositeRuntimeWorldCommit,
};
use crate::lifecycle::owner::RuntimeWorldOwnerConstructionContract;
use crate::publication::CompositeOwnerExecutionResults;

#[test]
fn constructor_binds_parent_basis_and_owner_results() {
    let fixture = component_fixture();
    let mut identities =
        RuntimeWorldOwnerConstructionContract::new().expect("World owner construction");
    let basis = admit_current(
        identities.issuer(),
        &fixture.relational_port,
        &fixture.signal_port,
        &fixture.correspondence_port,
        fixture.relational.clone(),
        fixture.signal.clone(),
        fixture.correspondence.clone(),
    )
    .expect("the real component tuple is admitted");
    let root = CompositeRuntimeWorldCommit::from_root_bootstrap(
        identities
            .issuer_mut()
            .composite_commit()
            .expect("root commit identity"),
        basis.clone(),
        identities
            .issuer_mut()
            .bootstrap_attempt()
            .expect("bootstrap provenance"),
        None,
    )
    .expect("root commit is coherent");
    let ordinary = CompositeRuntimeWorldCommit::from_ordinary_publication(
        identities
            .issuer_mut()
            .composite_commit()
            .expect("ordinary commit identity"),
        &root,
        basis.clone(),
        identities
            .issuer_mut()
            .publication_attempt()
            .expect("publication provenance"),
        &CompositeOwnerExecutionResults::retained(),
        None,
    )
    .expect("retained owner results match the exact predecessor basis");

    assert_eq!(
        ordinary.relational_change(),
        CompositeComponentChangePosture::RetainExact
    );
    assert_eq!(
        ordinary.signal_change(),
        CompositeComponentChangePosture::RetainExact
    );
    assert!(matches!(
        ordinary.parent(),
        crate::history::CompositeCommitParent::Ordinary(parent)
            if parent.commit() == root.identity()
    ));

    let foreign_fixture = component_fixture();
    let foreign_basis = admit_current(
        identities.issuer(),
        &foreign_fixture.relational_port,
        &foreign_fixture.signal_port,
        &foreign_fixture.correspondence_port,
        foreign_fixture.relational,
        foreign_fixture.signal,
        foreign_fixture.correspondence,
    )
    .expect("the same World owner can admit a distinct real component tuple");
    let denial = CompositeRuntimeWorldCommit::from_ordinary_publication(
        identities
            .issuer_mut()
            .composite_commit()
            .expect("mismatched commit identity"),
        &root,
        foreign_basis,
        identities
            .issuer_mut()
            .publication_attempt()
            .expect("mismatched publication provenance"),
        &CompositeOwnerExecutionResults::retained(),
        None,
    )
    .expect_err("retained evidence cannot be paired with a different component basis");
    assert!(matches!(
        denial,
        CompositeCommitConstructionDenial::BasisMismatch
    ));
}
