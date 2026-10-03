//! The installed marking ceiling bounds reader fan-out, never a legal writer.
//! Unwatched touch keys are not marking work; matched fan-out above the
//! ceiling publishes as a counted discontinuity that readers fully verify.

use super::super::logical_marking::NativeMarkingPrecision;
use super::super::mark_state::FullVerificationReason;
use super::*;
use crate::domain_computation::execution_runtime::product_world::{
    test_product_world_resources, WorthQueryProductWorldResources,
};
use crate::domain_computation::execution_runtime::{
    WorthQueryInvalidationResourceInstallation, WorthQueryInvalidationResources,
};
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world_with_product_resources, Account, AccountIdentity, AccountLabel,
    AuthorizationWorld,
};
use worth_relational::facade::{
    identity::PartitionId,
    symbols::ClientKey,
    transactions::{CreateIntent, EntitySpec},
};

/// One settlement registration is admitted under the same installed value and
/// spends about a thousand visits on its posting and index edits, so the
/// ceiling admits every registration; each journey crosses it by population.
const CEILING: u64 = 1_024;

fn ceiling_world() -> AuthorizationWorld {
    let (budgets, clock, defaults) = test_product_world_resources().into_parts();
    let invalidation =
        WorthQueryInvalidationResources::install(WorthQueryInvalidationResourceInstallation {
            maximum_marking_work: CEILING,
            ..defaults.installation()
        })
        .unwrap();
    installed_authorization_world_with_product_resources(WorthQueryProductWorldResources::new(
        budgets,
        clock,
        invalidation,
    ))
}

#[test]
fn unwatched_commit_with_more_touch_keys_than_the_ceiling_publishes_exactly() {
    let world = ceiling_world();
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let kind = graph
        .layout()
        .entity_kind(Account::reference().name())
        .unwrap();
    let locator = |entity, aspect, field| {
        graph
            .layout()
            .field_locator(entity, aspect, field)
            .unwrap()
            .clone()
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
        // Every created entity touches at least its three written fields.
        let batch = (0..CEILING).fold(
            WorkerIntentBatch::new("marking-ceiling-unwatched"),
            |batch, ordinal| {
                let value = |text: String| AspectValue::String(InternedString::Raw(text));
                batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: kind,
                    client_key: ClientKey::raw(format!("marking-ceiling-{ordinal}")),
                    fields: AspectFieldPatch::from(BTreeMap::from([
                        (
                            identity.clone(),
                            value(format!("marking-ceiling-{ordinal}")),
                        ),
                        (status.clone(), value("neutral".to_owned())),
                        (label.clone(), value("neutral".to_owned())),
                    ])),
                })))
            },
        );
        let committed = write_batch(runtime, batch);
        release_test_commit_snapshot(runtime, &committed);
        let (after_handle, after) = snapshot(runtime);
        let report = owner
            .native_marking_report(&after, &mut owner.edit_admission())
            .unwrap()
            .expect("the unwatched commit published its own delivery report");
        let NativeMarkingPrecision::Exact(counts) = report.precision else {
            panic!("unwatched touch keys must not degrade delivery");
        };
        assert!(
            counts.posting_key_lookups > CEILING,
            "the journey must probe more keys than the ceiling: {counts:?}"
        );
        runtime.snapshots().release_snapshot(&after_handle).unwrap();
    });
}

#[test]
fn matched_fan_out_above_the_ceiling_publishes_and_readers_fully_verify() {
    let world = ceiling_world();
    let selected = world.selected_product();
    let entity = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    let (_, product, _) = selected.into_parts();
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let status_ref = AccountStatus::reference();
    let status = graph
        .layout()
        .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let source = SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity().clone(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding: TypeId::of::<StatusOutput>(),
    };
    // A first match spends one visit plus two ordered reads and three ordered
    // edits, each at least one visit, so a quarter-ceiling of readers of one
    // field cannot be marked within the ceiling.
    let readers: Vec<_> = (0..(CEILING / 4) as usize)
        .map(|slot| RecordedSettlementIdentity::retain(&source, coordinate, slot))
        .collect();
    handle.with_runtime_mut(|runtime| {
        write_field(runtime, entity, status.clone(), "prime");
        let (before_handle, before) = snapshot(runtime);
        let facts: Arc<[_]> =
            Arc::from([field_fact(runtime, &before_handle, entity, status.clone())]);
        for reader in &readers {
            register(owner, reader.clone(), facts.clone(), &before, OrdSet::new());
        }
        write_field(runtime, entity, status, "closed");
        let (after_handle, after) = snapshot(runtime);
        let report = owner
            .native_marking_report(&after, &mut owner.edit_admission())
            .unwrap()
            .expect("the fanned-out commit published its own delivery report");
        let NativeMarkingPrecision::MarkingCeilingExceeded(counts) = report.precision else {
            panic!("fan-out above the ceiling must be reported, got {report:?}");
        };
        assert!(counts.matched_fact_postings > 0 && counts.matched_fact_postings <= CEILING);
        for reader in &readers {
            assert!(matches!(
                currentness(owner, &after, reader),
                SourceSettlementCurrentness::FullVerificationRequired(
                    FullVerificationReason::DeclaredChangeUnavailable
                )
            ));
        }
        assert!(
            matches!(
                currentness(owner, &before, &readers[0]),
                SourceSettlementCurrentness::Clean
            ),
            "the discontinuity cannot leak into the retained earlier snapshot"
        );
        for snapshot in [before_handle, after_handle] {
            runtime.snapshots().release_snapshot(&snapshot).unwrap();
        }
    });
}
