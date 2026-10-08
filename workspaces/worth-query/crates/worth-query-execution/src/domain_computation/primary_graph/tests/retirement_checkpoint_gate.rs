//! Fresh native admission cannot turn descriptive tombstones into producer provenance.
use super::fixture::{
    installed_authorization_world, live_scope, publish_relational_mutation, restored_world,
    AccountIdentity,
};
use crate::domain_computation::primary_graph::{
    application_attempt::{WorthQueryApplicationOutputPosture, WorthQueryCheckpointOutputRole},
    output_lineage::SealedNativeOutputWitness,
    WorthQueryApplicationObservedFact as Fact, WorthQueryApplicationOutputCorrespondence,
    WorthQueryPrincipalResolutionMode,
};
use crate::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;
use std::any::TypeId;
use worth_relational::facade::{
    identity::EntityId,
    transactions::{DeleteEntityIntent, EntityMutationIntent, MutationIntent, WorkerIntentBatch},
};

#[test]
fn fresh_native_retirement_descriptions_do_not_issue_original_output_authority() {
    let source = installed_authorization_world(true);
    let selected = source.selected_product();
    let resolve = |key: &str| {
        selected
            .resolve_entity(
                AccountIdentity::reference(),
                key.to_owned(),
                &live_scope(),
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap()
            .entity_id()
    };
    let original = resolve("account-2");
    let unrelated = resolve("account-1");
    drop(selected);
    for entity in [original, unrelated] {
        publish_relational_mutation(
            &source,
            WorkerIntentBatch::new("checkpoint-retirement-gate").push(MutationIntent::Entity(
                EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: entity }),
            )),
        );
    }
    let bytes = source
        .application
        .capture_application_checkpoint(CapturePolicy::SystemAllocation)
        .unwrap()
        .bytes()
        .to_vec();
    drop(source);
    let fresh =
        restored_world(super::super::WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes))
            .expect("actual opaque native checkpoint freshly installs");
    let selected = fresh.selected_product();
    let graph = fresh.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    handle.with_runtime(|runtime| {
        let snapshot = selected.application_basis().snapshot_handle();
        let read = runtime.read_truth();
        let view = read.project_snapshot(snapshot).unwrap();
        let original_metadata = view.entity_retirement(original).unwrap();
        let unrelated_metadata = view.entity_retirement(unrelated).unwrap();
        assert_eq!(original_metadata.kind_id(), unrelated_metadata.kind_id());
        assert_ne!(original_metadata.deleted_at_version(), unrelated_metadata.deleted_at_version());
        for entity in [original, unrelated] {
            let record = view.entity_retirement(entity).unwrap();
            let facts = [Fact::RetiredOutputEntity {
                entity_id: entity, kind: record.kind_id(), created_at: record.created_at_version(),
                deleted_at: record.deleted_at_version(), read_locator: "output-role:7:Account:7:retired".into(),
            }];
            assert!(facts[0].remains_equal_in(runtime, snapshot), "both actual tombstones match native truth");
            let correspondence = correspondence(entity);
            let candidate = SealedNativeOutputWitness::from_checkpoint_facts(&correspondence,
                graph.layout(), &facts, owner, &mut owner.edit_admission()).unwrap();
            let current = candidate.as_ref().is_some_and(|candidate| candidate.get().unwrap()
                .checkpoint_facts_current_in(runtime, snapshot, &facts, &mut owner.edit_admission()).unwrap());
            eprintln!("fresh installed retirement candidate entity={entity:?}, original={}, current={current}", entity == original);
            assert!(candidate.is_none() && !current, "native tombstone descriptions do not bind the original producer retirement");
        }
    });
}
fn correspondence(entity: EntityId) -> WorthQueryApplicationOutputCorrespondence {
    WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
        TypeId::of::<()>(),
        TypeId::of::<()>(),
        Default::default(),
        vec![WorthQueryCheckpointOutputRole {
            role: "retired".into(),
            posture: WorthQueryApplicationOutputPosture::Retire,
            entity_name: "Account".into(),
            entity,
        }],
        |_| Some(TypeId::of::<()>()),
    )
    .unwrap()
}
