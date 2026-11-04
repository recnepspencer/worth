//! The image preserves Query's original binding without a current-row proxy.
use super::*;
use worth_foundational::facade::{AspectKey, FieldKey};

#[test]
fn selected_home_image_preserves_original_binding_fields_and_world_publication() {
    let path = tempfile::tempdir().unwrap();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = ExecutionRequest::leased(&lease);
    let request = fixture::request(
        path.path().to_path_buf(),
        build(),
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
    );
    let writer = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("actual writer: {refused:?}"));
    let commit = match operations::set_key(writer.runtime(), 0x771, 8) {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => {
            receipt.commit_reference().commit_id
        }
        other => panic!("actual durable mutation: {other:?}"),
    };
    writer
        .close(&execution)
        .unwrap_or_else(|refused| panic!("writer close: {refused:?}"));
    with_bootstrap_advancement_from_request(execution, |phase| {
        let (query, home, source) = existing(path.path(), &phase);
        let native = home.relational().runtime();
        let authority = native.durability_authority();
        let subject = authority
            .discover_all_associated_effects(&execution)
            .unwrap()
            .into_iter()
            .find(|subject| subject.receipt().commit_id == commit)
            .expect("the original committed effect remains discoverable");
        let effect = authority
            .readmit_associated_effect(&execution, &subject)
            .expect("the exact selected image readmits the original effect");
        let kind = administration::kind(native, &execution, "worth-query-provider-idempotency");
        let entity = effect
            .semantic_result()
            .created_entities()
            .iter()
            .find(|created| created.key.kind == kind)
            .expect("the original commit creates its idempotency binding")
            .assigned;
        let row = effect.created_entity(&execution, entity).unwrap().unwrap();
        let aspect = AspectKey::new("idempotency").unwrap();
        for field in [
            "key",
            "intent",
            "outcome-identity",
            "emitted-effect-count",
            "product-affinity",
            "attempt-association-source",
            "output-settlement",
        ] {
            assert!(
                row.field(&aspect, &FieldKey::new(field).unwrap()).is_some(),
                "the original image preserves the binding field {field}"
            );
        }
        let publication = home
            .world()
            .observation_port()
            .readmitted_native_publication(&execution, effect)
            .expect("the same Native effect joins its original World occurrence");
        assert_eq!(publication.native_receipt().commit_id, commit);
        let port = native.owner_component_services().basis_port();
        let before = port
            .export_recorded_retained_component(
                &execution,
                publication.before().basis().relational_basis(),
            )
            .expect("the real predecessor basis remains exportable");
        let after = port
            .export_recorded_retained_component(
                &execution,
                publication.after().basis().relational_basis(),
            )
            .expect("the real successor basis remains exportable");
        assert_ne!(before, after);
        let reads = home.relational().cold_storage_reads();
        assert_eq!(
            native
                .repair_pending_publication_settlement(commit)
                .expect("receipt-only repair accepts actual image-readmitted custody"),
            *publication.native_receipt()
        );
        assert_eq!(
            home.relational().cold_storage_reads(),
            reads,
            "receipt-only repair must not demand a covered canonical body"
        );
        drop((before, after, publication, source, query));
        close_world(home, &execution);
    })
    .unwrap();
}
