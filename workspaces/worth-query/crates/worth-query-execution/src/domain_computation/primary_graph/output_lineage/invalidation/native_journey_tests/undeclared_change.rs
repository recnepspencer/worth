//! A commit delivered without touch keys declares no change. It starts the
//! delivery epoch under which every reader registered before it verifies in
//! full; a snapshot retained from before it keeps its exact answer, and a
//! reader registered after it is exact again.

use super::super::logical_marking::NativeMarkingPrecision;
use super::super::mark_state::FullVerificationReason;
use super::marking_ceiling::{unwatched_accounts, world_installing};
use super::*;
use crate::domain_computation::execution_runtime::WorthQueryInvalidationResourceInstallation;

/// Preparation memory that admits a registration and a one-field commit but
/// not the selectors of four thousand created entities.
const PREPARATION_BYTES: u64 = 128 * 1024;
const CREATED: u64 = 4_096;

#[test]
fn a_commit_delivered_without_touch_keys_starts_a_fully_verified_epoch() {
    let world = world_installing(|defaults| WorthQueryInvalidationResourceInstallation {
        maximum_preparation_bytes: PREPARATION_BYTES,
        ..defaults
    });
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
    let earlier = RecordedSettlementIdentity::retain(&source, coordinate, 0);
    let later = RecordedSettlementIdentity::retain(&source, coordinate, 1);
    let unwatched = unwatched_accounts(graph.layout(), CREATED, "undeclared-change");
    handle.with_open_runtime_mut(|runtime| {
        write_field(runtime, entity, status.clone(), "prime");
        let (before_handle, before) = snapshot(runtime);
        let facts: Arc<[_]> =
            Arc::from([field_fact(runtime, &before_handle, entity, status.clone())]);
        register(
            owner,
            earlier.clone(),
            facts.clone(),
            &before,
            OrdSet::new(),
        );

        // No selector of this commit matches the fact the reader holds; only
        // the delivery lost its keys.
        let committed = write_batch(runtime, unwatched);
        release_test_commit_snapshot(runtime, &committed);
        let (after_handle, after) = snapshot(runtime);
        let report = owner
            .native_marking_report(&after, &mut owner.edit_admission())
            .unwrap()
            .expect("the commit published its own delivery report");
        assert_eq!(
            report.precision,
            NativeMarkingPrecision::DeclaredChangeUnavailable,
            "selectors beyond preparation memory are delivered without keys"
        );
        assert!(matches!(
            currentness(owner, &after, &earlier),
            SourceSettlementCurrentness::FullVerificationRequired(
                FullVerificationReason::DeclaredChangeUnavailable
            )
        ));
        assert!(
            matches!(
                currentness(owner, &before, &earlier),
                SourceSettlementCurrentness::Clean
            ),
            "the discontinuity cannot leak into the retained earlier snapshot"
        );

        register(owner, later.clone(), facts, &after, OrdSet::new());
        assert!(
            matches!(
                currentness(owner, &after, &later),
                SourceSettlementCurrentness::Clean
            ),
            "a reader registered inside the new epoch is exact"
        );
        assert!(
            matches!(
                currentness(owner, &after, &earlier),
                SourceSettlementCurrentness::FullVerificationRequired(
                    FullVerificationReason::DeclaredChangeUnavailable
                )
            ),
            "the earlier reader stays behind the discontinuity until it is verified"
        );
        for snapshot in [before_handle, after_handle] {
            runtime.snapshots().release_snapshot(&snapshot).unwrap();
        }
    });
}
