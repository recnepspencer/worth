use std::sync::Arc;

use crate::tests::support::{create_entity_outcome, durable_runtime_with_test_schema};

#[test]
fn fresh_scalar_commit_seals_exact_native_lifecycle_and_field_touches() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use crate::history::data::RelationalDescriptiveTouch;
    use crate::identity::data::KindId;
    use crate::tests::support::test_execution_lease;

    for checked in [false, true] {
        let runtime = durable_runtime_with_test_schema();
        let committed = if checked {
            create_entity_outcome(
                &worth_execution::ExecutionRequest::leased(&test_execution_lease()),
                &runtime,
                "touch-value",
            )
        } else {
            create_entity_outcome(execution, &runtime, "touch-value")
        };
        let envelope = runtime
            .replay()
            .canonical_commit_envelope(execution, committed.commit.commit_id)
            .unwrap()
            .expect("performed commit has sealed envelope");
        let touches = envelope
            .descriptive_touches()
            .exact_touches()
            .expect("fresh scalar commit has exact native delta");
        assert!(touches.iter().any(|touch| matches!(
            touch,
            RelationalDescriptiveTouch::EntityLifecycle {
                kind: KindId(1),
                ..
            }
        )));
        assert!(touches.iter().any(|touch| matches!(touch,
            RelationalDescriptiveTouch::AspectRevision { aspect, .. } if aspect.as_str() == "name"
        )));
        assert!(touches.iter().any(|touch| matches!(touch,
            RelationalDescriptiveTouch::FieldRevision { kind: KindId(1), aspect, path, presence, .. }
                if aspect.as_str() == "name"
                    && path.fields()[0].as_str() == "name"
                    && *presence == crate::storage::data::RelationalFieldPresence::Present
        )));
    }
}

#[test]
fn installed_field_index_commit_touch_carries_native_old_and_new_keys() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use crate::history::data::RelationalDescriptiveTouch;
    use crate::indexes::data::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};
    use crate::tests::support::{aspect_field_locator, aspect_key, field_key};
    use worth_foundational::facade::AspectValue;

    let runtime = durable_runtime_with_test_schema();
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "touch-index".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
            branch_scoped: false,
        })
        .expect("index fixture has identity capacity");
    let committed = create_entity_outcome(execution, &runtime, "indexed-value");
    let envelope = runtime
        .replay()
        .canonical_commit_envelope(execution, committed.commit.commit_id)
        .unwrap()
        .expect("performed indexed commit has envelope");
    let touches = envelope
        .descriptive_touches()
        .exact_touches()
        .expect("exact native graph");
    let expected = crate::storage::data::AuthoritativeFieldComparisonKey::from_aspect_value(
        &AspectValue::String("indexed-value".into()),
    );
    assert!(touches.iter().any(|touch| matches!(touch,
        RelationalDescriptiveTouch::IndexMembership { index: id, old_key: None, new_key: Some(key), .. }
            if *id == index.index_id && key == &expected
    )));
}

use super::{RelationalCommitArtifact, RelationalCommitArtifactDenial};

#[test]
fn recovery_relinks_an_already_sealed_payload_without_reencoding_or_weakening_root_checks() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = crate::tests::support::runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "preencoded-recovery-root");
    let root = runtime
        .history
        .branch_cell(&crate::history::data::BranchId("main".to_owned()))
        .and_then(|cell| cell.root())
        .expect("committed main has one root");
    let envelope = root
        .canonical_envelope()
        .cloned()
        .expect("committed root carries its canonical envelope");
    let preencoded = RelationalCommitArtifact::from_envelope(Arc::clone(&envelope)).unwrap();
    let legacy =
        RelationalCommitArtifact::from_envelope_with_root(Arc::clone(&envelope), Arc::clone(&root))
            .unwrap();
    let relinked = preencoded
        .relink_preencoded_recovery(&envelope, Some(&root), None)
        .unwrap();
    assert!(Arc::ptr_eq(
        &preencoded.envelope_residency(),
        &relinked.envelope_residency()
    ));
    assert_eq!(relinked.identity(), legacy.identity());
    assert_eq!(relinked.parentage(), legacy.parentage());
    assert_eq!(relinked.roots(), legacy.roots());
    assert_eq!(
        relinked
            .canonical_payload_digest()
            .expect("resident fixture has a canonical digest"),
        legacy
            .canonical_payload_digest()
            .expect("resident fixture has a canonical digest")
    );

    let equal_but_unsealed = Arc::new(envelope.copy_original_for_test(execution));
    assert!(matches!(
        preencoded.relink_preencoded_recovery(&equal_but_unsealed, Some(&root), None),
        Err(RelationalCommitArtifactDenial::RootLinkage)
    ));
    let mismatched_schema = crate::branch::RelationalBranchRootDescriptor::new(
        *legacy.roots().truth_root(),
        [0xAB; 32],
    );
    assert!(matches!(
        preencoded.relink_preencoded_recovery(&envelope, None, Some(&mismatched_schema)),
        Err(RelationalCommitArtifactDenial::RootLinkage)
    ));
}
