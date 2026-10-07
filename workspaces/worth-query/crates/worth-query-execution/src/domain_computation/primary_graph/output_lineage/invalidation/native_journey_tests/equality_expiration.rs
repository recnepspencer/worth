//! Equality recovery uses native revisions, never the expired origin's facts.
use crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts;

use super::super::{ConsumedOutputCurrentness, FullVerificationReason};
use super::*;
use crate::domain_computation::execution_runtime::WorthQueryInvalidationResourceInstallation;
use crate::domain_computation::primary_graph::{
    invariant_projection::{ConsumedOutputEvidence, ConsumedOutputVerification},
    output_lineage::input_cutoff::StableEqualityConsequence,
};

const WINDOW: usize = 4;

#[test]
fn an_expired_chain_origin_does_not_block_a_fresh_terminal() {
    chain_expiration(false, false);
}

#[test]
fn an_expired_chain_terminal_is_compared_and_reestablished_or_changed() {
    chain_expiration(true, false);
}

#[test]
fn an_expired_chain_detects_a_changed_output_with_an_unchanged_source() {
    chain_expiration(true, true);
}

fn chain_expiration(expire_terminal: bool, output_only: bool) {
    let world = super::marking_ceiling::world_installing(|defaults| {
        WorthQueryInvalidationResourceInstallation {
            maximum_retained_positions: WINDOW,
            ..defaults
        }
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
    let source = SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding: TypeId::of::<StatusOutput>(),
    };
    let origin = RecordedSettlementIdentity::retain(&source, coordinate, 0);
    let terminal = RecordedSettlementIdentity::retain(&source, coordinate, 1);
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let status_ref = AccountStatus::reference();
    let label_ref = AccountLabel::reference();
    let status = graph
        .layout()
        .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let label = graph
        .layout()
        .field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    let note_ref =
        crate::domain_computation::primary_graph::tests::fixture::AccountNote::reference();
    let note = graph
        .layout()
        .field_locator(note_ref.entity(), note_ref.aspect(), note_ref.field())
        .unwrap()
        .clone();
    let churn = if output_only { &note } else { &label };
    handle.with_runtime_mut(|runtime| {
        write_field(runtime, entity, label.clone(), "prime");
        if output_only {
            write_field(runtime, entity, note.clone(), "prime");
        }
        let (origin_handle, origin_basis) = snapshot(runtime);
        let origin_fact = field_fact(runtime, &origin_handle, entity, status.clone());
        // This actor fixture's output is the existing live native entity.
        // Its projection is an actual native liveness fact, not a synthetic stamp.
        let mut admission = owner.edit_admission();
        let output = super::super::output_facts::RegisteredOutputFacts {
            facts: Arc::from([if output_only {
                field_fact(runtime, &origin_handle, entity, label.clone())
            } else {
                WorthQueryApplicationObservedFact::SourceEntity { entity_id: entity }
            }]),
            _capacity: super::super::retention::reserve(
                &owner.resources,
                std::mem::size_of::<WorthQueryApplicationObservedFact>() as u64,
                &mut admission,
            )
            .unwrap(),
        };
        owner
            .register_settlement(
                SettlementRegistration {
                    work_membership: None,
                    identity: Arc::clone(&origin),
                    facts: RetainedSourceFacts::for_test(false, Arc::from([origin_fact.clone()])),
                    output_facts: Some(output),
                    read_basis: origin_basis.clone(),
                    stale_at_read_basis: OrdSet::new(),
                    requirement: None,
                    upstream: OrdSet::new(),
                },
                &mut admission,
            )
            .unwrap();
        write_field(runtime, entity, status.clone(), "closed");
        if !expire_terminal {
            expire(runtime, entity, churn);
        }
        let (terminal_handle, terminal_basis) = snapshot(runtime);
        let prepared = owner
            .prepare_current_stable_settlement(
                SettlementRegistration {
                    work_membership: None,
                    identity: Arc::clone(&terminal),
                    facts: RetainedSourceFacts::for_test(false, Arc::from([field_fact(
                        runtime,
                        &terminal_handle,
                        entity,
                        status.clone(),
                    )])),
                    output_facts: None,
                    read_basis: terminal_basis.clone(),
                    stale_at_read_basis: OrdSet::new(),
                    requirement: None,
                    upstream: OrdSet::new(),
                },
                &terminal_basis,
                StableEqualityConsequence::native_actor_fixture(
                    Arc::clone(&origin),
                    Arc::clone(&terminal),
                    &terminal_basis,
                ),
                &mut owner.edit_admission(),
            )
            .unwrap();
        assert!(prepared.install().is_ok());
        if expire_terminal {
            expire(runtime, entity, churn);
        }
        let (current_handle, current_basis) = snapshot(runtime);
        assert!(matches!(
            currentness(owner, &current_basis, &origin),
            SourceSettlementCurrentness::FullVerificationRequired(
                FullVerificationReason::RetainedDeliveryGap
            )
        ));
        let verify = |runtime: &RelationalRuntime, snapshot, basis| {
            ConsumedOutputEvidence::verify_candidate_with_admission(
                &origin,
                &crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts::for_test(false, Arc::from([origin_fact.clone()])).for_comparison().unwrap(),
                &[],
                None,
                &origin_basis,
                owner,
                runtime,
                snapshot,
                basis,
                &mut owner.edit_admission(),
            )
        };
        assert_eq!(
            verify(runtime, &current_handle, &current_basis),
            Ok(ConsumedOutputVerification::Current),
            "the terminal's native facts replaced the origin's changed source"
        );
        assert!(
            matches!(
                owner
                    .consumed_output_currentness(
                        &current_basis,
                        &origin,
                        &mut owner.edit_admission()
                    )
                    .unwrap(),
                ConsumedOutputCurrentness::CanonicallyEqualClean(_)
            ),
            "the next reader uses the re-established terminal's marks"
        );
        if expire_terminal {
            write_field(
                runtime,
                entity,
                if output_only {
                    label.clone()
                } else {
                    status.clone()
                },
                "changed-terminal",
            );
            expire(runtime, entity, churn);
            let (changed_handle, changed_basis) = snapshot(runtime);
            assert_eq!(
                verify(runtime, &changed_handle, &changed_basis),
                Ok(ConsumedOutputVerification::ChangedUpstream)
            );
            runtime
                .snapshots()
                .release_snapshot(&changed_handle)
                .unwrap();
        }
        for snapshot in [origin_handle, terminal_handle, current_handle] {
            runtime.snapshots().release_snapshot(&snapshot).unwrap();
        }
    });
}

fn expire(runtime: &mut RelationalRuntime, entity: EntityId, label: &AspectFieldLocator) {
    for index in 0..=WINDOW {
        write_field(
            runtime,
            entity,
            label.clone(),
            if index % 2 == 0 { "aa" } else { "bb" },
        );
    }
}
