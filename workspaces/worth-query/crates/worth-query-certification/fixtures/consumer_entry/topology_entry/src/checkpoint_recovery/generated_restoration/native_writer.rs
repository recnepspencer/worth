//! A native Relational writer that is not Query moves a generated output.

use super::*;

/// A Ready whose output moved under current source facts is refreshed under
/// the Preserve posture. In this family another producer declares it: the
/// refresh switches to that producer, which runs once over the live output.
#[test]
fn a_refresh_that_must_switch_producers_preserves_the_live_output() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    drop(settle(&request, &application));
    let ring = ring(&request);

    let branch = application.current_world();
    let record = application
        .on_branch(branch)
        .select()
        .unwrap()
        .resolve_entity(
            BodyKey::reference::<CheckpointSchema>(),
            ring[1].0.clone(),
            &scope,
            primary_graph::WorthQueryPrincipalResolutionMode::Certification,
        )
        .unwrap()
        .relational_record_identity_parts();
    application
        .publish_native_field_write_for_test(
            branch,
            record,
            Length::reference::<CheckpointSchema>(),
            length(44),
            &scope,
        )
        .expect("the native-writer fixture requires an open application owner");

    assert_eq!(redemanded(&request, &application), (Posture::Performed, 1));
    assert_eq!(super::ring(&request), ring);
    assert_eq!(redemanded(&request, &application), (Posture::Performed, 0));
    assert_eq!(super::ring(&request), ring);
}
