//! Cold expected versions must survive selection without contacting healthy producers.
use super::*;
fn healthy(worker: bool) {
    let (checkpoint, original) = build_change::old_checkpoint();
    let host = world::restore_same_build(checkpoint, worker);
    assert_eq!(world::outputs(&host, host.current_world()), original);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let (document, before) = build_change::record_edit(&host, &request);
    let current = document.selection(&request).unwrap();
    let document = courts::configure_all_trees(&host, document);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let projection = source_projection::Observation::begin();
    let selected = courts::checkout(&document, &request, before, current, 0xf057_11)
        .unwrap_or_else(|cause| panic!("{cause:?}"));
    assert_eq!(
        world::outputs(&host, selected.physical_endpoint().product_branch()),
        original
    );
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    assert_eq!(projection.reads(), 0);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
#[test]
fn cold_same_build_selection_retains_both_families_without_contacts() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    healthy(false);
}
#[test]
fn worker_only_cold_same_build_selection_retains_both_families_without_contacts() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    healthy(true);
}

/// Change one actual recorded Query role ID while retaining the native frame,
/// producer, source, implementation and installed declaration. The public import
/// accepts descriptive metadata, but selection must bind it to Native's roles.
fn corrupted_subject(worker: bool) -> WorthQueryHistoryPublicationCorrespondenceMismatch {
    let (checkpoint, original) = build_change::old_checkpoint();
    let checkpoint = checkpoint.rewrite_authenticated_checkpoint_body_for_test(|body| {
        fn skip_text(body: &[u8], at: &mut usize) {
            let count = u64::from_be_bytes(body[*at..*at + 8].try_into().unwrap()) as usize;
            *at += 8 + count;
        }
        let native = u64::from_be_bytes(body[14..22].try_into().unwrap()) as usize;
        let mut at = 30 + native;
        skip_text(body, &mut at); // producer
        at += 1 + 48 + 32 + 16 + 32 + 1 + 32 + 32;
        let roles = u64::from_be_bytes(body[at..at + 8].try_into().unwrap());
        assert!(roles > 0);
        at += 8;
        skip_text(body, &mut at); // role
        at += 1; // posture
        skip_text(body, &mut at); // entity declaration
                                  // An impossible generation is enough: declaration meaning is intact,
                                  // but this record was never owned by the selected publication.
        body[at + 12..at + 16].copy_from_slice(&u32::MAX.to_be_bytes());
    });
    let host = world::restore_same_build(checkpoint, worker);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let (document, before) = build_change::record_edit(&host, &request);
    let current = document.selection(&request).unwrap();
    let branch = current.physical_endpoint().product_branch();
    let model = world::model(&host, branch);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let document = courts::configure_all_trees(&host, document);
    let refusal = courts::checkout(&document, &request, before, current.clone(), 0xf057_12)
        .expect_err("unowned role identity cannot cross the transport boundary");
    let (
        _,
        WorthQueryHistoryNavigationAdvanceDenial::Outputs(
            WorthQueryHistoryOutputDenial::Navigation(
                WorthQueryHistoryNavigationDenial::OutputPublicationCorrespondence(publication),
            ),
        ),
    ) = refusal
    else {
        panic!("{refusal:?}");
    };
    let publication: WorthQueryHistoryPublicationCorrespondenceMismatch = publication;
    assert_eq!(publication.source(), model.native_id);
    assert_eq!(publication.producer(), "owned-outputs.first.initial");
    assert_eq!(publication.family(), "first");
    assert_eq!(document.selection(&request).unwrap(), current);
    assert_eq!(world::model(&host, branch), model);
    assert_eq!(world::outputs(&host, branch), original);
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
    publication
}

fn refused_subject_survives_host_without_custody(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let capture = WorthQueryHistoryPublicationCorrespondenceMismatch::observe_capture_for_test();
    let subject = corrupted_subject(worker);
    let twin = subject.clone();
    assert!(
        capture.captured_for_test(),
        "the actual cold Native basis was observed"
    );
    assert!(
        !capture.basis_alive_for_test(),
        "retaining a refusal must not retain its Native basis after host teardown"
    );
    assert_eq!(twin.producer(), "owned-outputs.first.initial");
    assert_eq!(twin.family(), "first");
    assert_eq!(twin.source(), subject.source());
    assert_eq!(
        twin.publication_coordinate(),
        subject.publication_coordinate()
    );
}
#[test]
fn cold_role_outside_the_selected_publication_refuses_without_effect() {
    refused_subject_survives_host_without_custody(false);
}
#[test]
fn worker_only_cold_refusal_keeps_data_without_native_custody() {
    refused_subject_survives_host_without_custody(true);
}
