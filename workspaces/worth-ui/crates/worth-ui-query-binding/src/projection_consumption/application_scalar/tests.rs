use super::UiApplicationScalarProjectionFactReceipt;
use crate::{UiApplicationScalarProjectionRegistration, WorthUiStatusSourceOwner};

#[test]
fn projection_evidence_matches_original_publications_across_edits_and_foreign_owners() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<crate::UiProjectionObservation>();
    send_sync::<UiApplicationScalarProjectionRegistration>();
    let owner = WorthUiStatusSourceOwner::install().unwrap();
    let foreign = WorthUiStatusSourceOwner::install().unwrap();
    let original = owner.read_status().unwrap();
    let registration = UiApplicationScalarProjectionRegistration::query_issued(&original).unwrap();
    let installed = original.query_receipt().inspect();
    let installed_query = installed.query_identity().to_owned();
    let installed_parameters = installed.parameter_binding_identity().to_owned();
    let installed_basis = installed.basis().clone();
    let installed_projection = original.value().identity.clone();
    let edited = owner
        .publish_source(
            crate::WorthUiScalarProjectionSourceRecord::new("ONLINE", 1).unwrap(),
            std::num::NonZeroUsize::new(64),
        )
        .unwrap();
    let other = foreign.read_status().unwrap();
    for publication in [original, edited, other] {
        let receipt = publication.query_receipt();
        let inspection = receipt.inspect();
        let basis = inspection.basis().clone();
        let released = inspection.terminal_resources_released();
        let parameters = inspection.parameter_binding_identity().to_owned();
        let value = publication.value().clone();
        let expected_admission = value.identity == installed_projection
            && inspection.query_identity() == installed_query
            && parameters == installed_parameters
            && basis.runtime_instance() == installed_basis.runtime_instance()
            && basis.branch() == installed_basis.branch();
        let fact = UiApplicationScalarProjectionFactReceipt::query_issued(publication).unwrap();
        assert_eq!(fact.publication_basis(), &basis);
        assert_eq!(fact.owner_order(), basis.version());
        assert_eq!(fact.value(), &value);
        assert_eq!(fact.terminal_resources_released(), released);
        assert_eq!(
            fact.publication_identity().parameter_binding_identity(),
            parameters
        );
        assert_eq!(registration.admits(&fact), expected_admission);
    }
    drop(registration);
    assert!(owner.close().owner_terminal());
    assert!(foreign.close().owner_terminal());
}
