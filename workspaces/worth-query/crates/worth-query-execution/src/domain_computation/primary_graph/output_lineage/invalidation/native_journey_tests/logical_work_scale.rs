//! Logical marking owner proof with real Native rows and an independent
//! field-revision oracle. This does not measure public producer contacts.
use super::super::logical_marking::{LogicalMarkingCounts, NativeMarkingPrecision};
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{Account, AccountIdentity};
use worth_relational::facade::{
    identity::PartitionId,
    symbols::ClientKey,
    transactions::{CreateIntent, CreatedEntityRef, EntitySpec},
};

#[test]
fn neutral_native_population_keeps_logical_marking_local_and_matches_revision_oracle() {
    let mut expected_counts = None;
    for population in [1, 10, 100, 1000] {
        let counts = run_population(population);
        assert_eq!(counts.marked_fact_ordinals, 1);
        assert_eq!(counts.visited_vertices, 2);
        assert_eq!(counts.downstream_edges, 1);
        assert!(counts.matched_fact_postings >= counts.marked_fact_ordinals);
        if let Some(expected) = expected_counts {
            assert_eq!(
                counts, expected,
                "neutral population {population} changed logical work"
            );
        } else {
            expected_counts = Some(counts);
        }
    }
}

fn run_population(population: usize) -> LogicalMarkingCounts {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let (_, product, _) = selected.into_parts();
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let kind = graph
        .layout()
        .entity_kind(Account::reference().name())
        .unwrap();
    let locator = |entity, aspect, field| {
        let installed = graph.layout().field_locator(entity, aspect, field).unwrap();
        // The independent oracle reads Native authority, matching the field
        // meaning rather than the layout's effective projection posture.
        AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            installed.aspect().aspect_key().clone(),
            installed.field_path().clone(),
        )
    };
    let identity_ref = AccountIdentity::reference();
    let status_ref = AccountStatus::reference();
    let label_ref = AccountLabel::reference();
    let identity = locator(
        identity_ref.entity(),
        identity_ref.aspect(),
        identity_ref.field(),
    );
    let status = locator(status_ref.entity(), status_ref.aspect(), status_ref.field());
    let label = locator(label_ref.entity(), label_ref.aspect(), label_ref.field());
    handle.with_runtime_mut(|runtime| {
        let references: Vec<_> = (0..population).map(|ordinal| CreatedEntityRef {
            partition_id: PartitionId::main(), kind_id: kind,
            client_key: ClientKey::raw(format!("logical-marking-{ordinal}")),
        }).collect();
        let batch = references.iter().enumerate().fold(
            WorkerIntentBatch::new("logical-marking-neutral-population"),
            |batch, (ordinal, reference)| {
                let value = |text: String| AspectValue::String(InternedString::Raw(text));
                batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: reference.partition_id, kind_id: reference.kind_id,
                    client_key: reference.client_key.clone(),
                    fields: AspectFieldPatch::from(BTreeMap::from([
                        (identity.clone(), value(format!("logical-marking-{ordinal}"))),
                        (status.clone(), value("neutral".to_owned())),
                        (label.clone(), value("before".to_owned())),
                    ])),
                })))
            },
        );
        let committed = write_batch(runtime, batch);
        let entities: Vec<_> = references.iter().map(|reference|
            committed.created_entity(reference).expect("actual Native create correspondence")).collect();
        release_test_commit_snapshot(runtime, &committed);
        let (before_handle, before) = snapshot(runtime);
        let source = |entity, binding| SemanticSource {
            runtime_authority: world.application.runtime.authority_identity().as_u64(),
            schema: world.application.installed_schema.binding_identity().clone(),
            scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
            output_binding: binding,
        };
        let identities: Vec<_> = entities.iter().map(|entity|
            RecordedSettlementIdentity::retain(&source(*entity, TypeId::of::<LabelOutput>()), coordinate, 0)).collect();
        // This oracle reads Native revisions directly; it does not call the
        // marking verifier or derive expected ordinals from actor postings.
        let old_revisions: Vec<_> = entities.iter().map(|entity| {
            let truth = runtime.read_truth().project_snapshot(&before_handle).unwrap();
            [truth.entity_field_revision(*entity, &label).unwrap(),
             truth.entity_field_revision(*entity, &status).unwrap()]
        }).collect();
        for (entity, output) in entities.iter().zip(&identities) {
            register(owner, output.clone(), Arc::from([
                field_fact(runtime, &before_handle, *entity, label.clone()),
                field_fact(runtime, &before_handle, *entity, status.clone()),
            ]), &before, OrdSet::new());
            assert!(owner.resources.retained_capacity_bytes()
                <= owner.resources.installation().maximum_retained_bytes,
                "actual Native registration must stay within the installed capacity");
        }
        let downstream = RecordedSettlementIdentity::retain(
            &source(entities[0], TypeId::of::<DownstreamOutput>()), coordinate, 0,
        );
        register(owner, downstream.clone(), Arc::from([]), &before,
            OrdSet::unit(identities[0].clone()));
        write_field(runtime, entities[0], label.clone(), "after");
        let (after_handle, after) = snapshot(runtime);
        let report = owner.native_marking_report(&after, &mut owner.edit_admission())
            .unwrap().expect("exact Native publication owns a delivery report");
        let NativeMarkingPrecision::Exact(counts) = report.precision else {
            panic!("the actual declared field change must have exact logical counts");
        };
        for ((entity, output), old) in entities.iter().zip(&identities).zip(old_revisions) {
            let truth = runtime.read_truth().project_snapshot(&after_handle).unwrap();
            let now = [truth.entity_field_revision(*entity, &label).unwrap(),
                truth.entity_field_revision(*entity, &status).unwrap()];
            let changed: OrdSet<_> = old.iter().zip(now).enumerate()
                .filter_map(|(ordinal, (old, now))| (*old != now).then_some(ordinal)).collect();
            match currentness(owner, &after, output) {
                SourceSettlementCurrentness::Clean => assert!(changed.is_empty()),
                SourceSettlementCurrentness::Dirty(ordinals) => {
                    assert!(!changed.is_empty());
                    assert_eq!(ordinals, changed);
                }
                _ => panic!("direct field output must match the independent Native oracle"),
            }
        }
        assert!(matches!(currentness(owner, &after, &downstream),
            SourceSettlementCurrentness::PendingUpstream(edges) if edges == OrdSet::unit(identities[0].clone())));
        assert!(owner.native_marking_report(&before, &mut owner.edit_admission()).unwrap().is_none(),
            "a historical image cannot report the newer Native delivery");
        let late = RecordedSettlementIdentity::retain(
            &source(entities[0], TypeId::of::<LateOutput>()), coordinate, 0,
        );
        register(owner, late, Arc::from([field_fact(runtime, &after_handle, entities[0], label.clone())]),
            &after, OrdSet::new());
        assert_eq!(owner.native_marking_report(&after, &mut owner.edit_admission()).unwrap(), Some(report),
            "a derived registration preserves the original delivery report");
        for snapshot in [before_handle, after_handle] {
            runtime.snapshots().release_snapshot(&snapshot).unwrap();
        }
        counts
    })
}
