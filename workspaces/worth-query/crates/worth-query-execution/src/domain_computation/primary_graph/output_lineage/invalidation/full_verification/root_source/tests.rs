//! Completeness of retained upstream evidence against actual actor registration.

use super::*;
use std::any::TypeId;

use crate::domain_computation::primary_graph::{
    invariant_projection::ConsumedOutputEvidence,
    output_lineage::{ProductCoordinate, SemanticSource},
    tests::fixture::{installed_authorization_world, live_scope, AccountStatus},
    WorthQueryPrincipalResolutionMode,
};

struct UpstreamOutput;
struct ConsumerOutput;

#[test]
fn retained_source_must_include_every_registered_upstream_edge() {
    let world = installed_authorization_world(true);
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
    let source = |output_binding| {
        SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding,
    }
    };
    let upstream =
        RecordedSettlementIdentity::retain(&source(TypeId::of::<UpstreamOutput>()), coordinate, 0);
    let consumer =
        RecordedSettlementIdentity::retain(&source(TypeId::of::<ConsumerOutput>()), coordinate, 0);
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    handle.with_runtime_mut(|runtime| {
        let basis = runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap();
        let snapshot = runtime
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap();
        let selected = Arc::new(runtime.read_truth().positioned_snapshot(&snapshot).unwrap());
        for (identity, consumed_upstream) in [
            (Arc::clone(&upstream), OrdSet::new()),
            (Arc::clone(&consumer), OrdSet::unit(Arc::clone(&upstream))),
        ] {
            owner
                .register_settlement(
                    super::super::super::SettlementRegistration {
                        work_membership: None,
                        identity,
                        facts: Arc::from([]),
                        output_facts: None,
                        read_basis: (*selected).clone(),
                        stale_at_read_basis: im::OrdSet::new(),
                        requirement: None,
                        upstream: consumed_upstream,
                    },
                    &mut owner.edit_admission(),
                )
                .unwrap();
        }
        let retained = |identity, upstream| {
            ConsumedOutputEvidence::retained_for_test(
                owner,
                identity,
                Arc::from([]),
                upstream,
                None,
                Arc::clone(&selected),
            )
        };
        let omitted = retained(Arc::clone(&consumer), Vec::new());
        let complete = retained(Arc::clone(&consumer), vec![retained(upstream, Vec::new())]);
        let captured = owner
            .capture_full_verification(runtime, &snapshot, &selected, &mut owner.edit_admission())
            .unwrap();
        assert_eq!(
            captured.require_complete_root_upstream(
                EvidenceView::from(&omitted),
                &consumer,
                &mut owner.edit_admission(),
            ),
            Err(FullVerificationStop::OutputEvidenceUnavailable),
        );
        assert_eq!(
            captured.require_complete_root_upstream(
                EvidenceView::from(&complete),
                &consumer,
                &mut owner.edit_admission(),
            ),
            Ok(()),
            "the control carries the actual registered upstream identity",
        );
        drop(captured);
        drop((omitted, complete, selected));
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
    });
}
