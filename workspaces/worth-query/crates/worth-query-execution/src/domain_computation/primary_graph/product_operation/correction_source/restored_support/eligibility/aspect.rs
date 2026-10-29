//! Native whole-aspect witnesses select real field writers, including no-ops.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, live_scope, publish_relational_mutation, AccountIdentity,
    AccountLabel,
};
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, CanonicalFieldPath, FieldKey, LocatorAuthority,
};
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

#[test]
fn whole_aspect_support_selects_field_noops_and_compares_complete_content() {
    let world = installed_authorization_world(true);
    world
        .application
        .with_host_advancement(|phase| {
            let execution = phase.borrowed_request_for_test();
            let host = phase.test_host_request();
            let custody = host.phase_for_application(&world.application);
            let original = world.selected_product(&host);
            let entity = original
                .resolve_entity(
                    &custody,
                    AccountIdentity::reference(),
                    "account-2".into(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id();
            let field = AccountLabel::reference();
            let aspect = AspectKey::new(field.aspect()).unwrap();
            let locator = AspectFieldLocator::new(
                LocatorAuthority::Authoritative,
                aspect.clone(),
                CanonicalFieldPath::single(FieldKey::new(field.field()).unwrap()),
            );
            let write = |value: &str| {
                WorkerIntentBatch::new("aspect-field-writer").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: entity,
                        fields: AspectFieldPatch::new(
                            [(locator.clone(), AspectValue::String(value.into()))]
                                .into_iter()
                                .collect(),
                        ),
                    }),
                ))
            };
            publish_relational_mutation(&execution, &world, write("secondary"));
            let equal = world.selected_product(&host);
            publish_relational_mutation(&execution, &world, write("changed"));
            let different = world.selected_product(&host);
            publish_relational_mutation(&execution, &world, write("secondary"));
            let restored = world.selected_product(&host);
            let graph = world.application.runtime.primary_graph().unwrap();
            graph
                .integration_handle()
                .with_runtime(&execution, |runtime| {
                    let source_snapshot = original.application_basis().snapshot_handle();
                    let target_snapshot = restored.application_basis().snapshot_handle();
                    let revision = runtime
                        .read_truth()
                        .read_exact_snapshot_entity_aspect_version(
                            &execution,
                            source_snapshot,
                            entity,
                            &aspect,
                        )
                        .unwrap()
                        .unwrap();
                    let fact = Fact::SourceAspectRevision {
                        entity_id: entity,
                        aspect: aspect.clone(),
                        native_revision: revision,
                    };
                    assert_ne!(
                        revision,
                        runtime
                            .read_truth()
                            .read_exact_snapshot_entity_aspect_version(
                                &execution,
                                target_snapshot,
                                entity,
                                &aspect,
                            )
                            .unwrap()
                            .unwrap(),
                        "actual revision changed despite restored complete content"
                    );
                    let facts: FactPayload =
                        std::sync::Arc::<[Fact]>::from(vec![fact.clone()]).into();
                    let before = runtime
                        .read_truth()
                        .project_snapshot(source_snapshot)
                        .unwrap();
                    let after = runtime
                        .read_truth()
                        .project_snapshot(target_snapshot)
                        .unwrap();
                    let selection = overlap::select(execution, &facts, &before, &after).unwrap();
                    let keys = selection.keys.iter().cloned().collect::<Vec<_>>();
                    let basis = restored.product().relational_basis();
                    let selected = runtime
                        .select_exact_commit(
                            &basis.observation(),
                            basis.observation().commit_id().unwrap(),
                        )
                        .into_outcome()
                        .unwrap();
                    let mut cursor = runtime
                        .authored_write_history_cursor(
                            execution,
                            original.product().relational_basis(),
                            selected,
                            &keys,
                        )
                        .unwrap();
                    let page = cursor
                        .next_page(
                            runtime,
                            execution,
                            RelationalAuthoredWritePageLimits {
                                maximum_entries: NonZeroUsize::new(16).unwrap(),
                                maximum_source_visits: NonZeroUsize::new(64).unwrap(),
                                maximum_owned_bytes: NonZeroUsize::new(usize::MAX).unwrap(),
                            },
                        )
                        .unwrap();
                    assert!(page.is_complete());
                    assert_eq!(
                        page.entries()
                            .map(|entry| entry.commit_id())
                            .collect::<Vec<_>>(),
                        [&equal, &different, &restored].map(|selected| selected
                            .product()
                            .relational_basis()
                            .observation()
                            .commit_id()
                            .unwrap()),
                        "all authored occurrences survive, even equal writes and restored content"
                    );
                    compare(runtime, execution, source_snapshot, target_snapshot, &facts).unwrap();
                    assert!(matches!(
                        compare(
                            runtime,
                            execution,
                            source_snapshot,
                            different.application_basis().snapshot_handle(),
                            &facts
                        ),
                        Err(Denial::SourceDiverged)
                    ));
                    // A captured revision is still checked at the ORIGINAL root.
                    let forged: FactPayload =
                        std::sync::Arc::<[Fact]>::from(vec![Fact::SourceAspectRevision {
                            entity_id: entity,
                            aspect: aspect.clone(),
                            native_revision: None,
                        }])
                        .into();
                    assert!(revision.is_some());
                    assert!(matches!(
                        compare(
                            runtime,
                            execution,
                            source_snapshot,
                            target_snapshot,
                            &forged
                        ),
                        Err(Denial::SourceDiverged)
                    ));
                });
        })
        .unwrap();
}
