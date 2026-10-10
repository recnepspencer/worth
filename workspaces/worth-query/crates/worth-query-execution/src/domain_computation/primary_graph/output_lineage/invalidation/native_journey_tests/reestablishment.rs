//! A fully compared expired closure clears delivery-only pending edges upstream-first.
use super::*;
use crate::domain_computation::execution_runtime::WorthQueryInvalidationResourceInstallation;
use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
use crate::domain_computation::primary_graph::tests::fixture::{
    AccountBlocked, PrincipalIdentityField,
};
use worth_relational::facade::{
    identity::PartitionId,
    symbols::ClientKey,
    transactions::{
        CreateIntent, CreatedRelationRef, DeleteRelationIntent, EntityReference,
        RelationMutationIntent, RelationSpec,
    },
};

#[test]
fn full_comparison_clears_expired_pending_edges_only_after_the_upstream_is_clean() {
    const WINDOW: usize = 4;
    let world = super::marking_ceiling::world_installing(|defaults| {
        WorthQueryInvalidationResourceInstallation {
            maximum_retained_positions: WINDOW,
            ..defaults
        }
    });
    let selected = world.selected_product();
    let account = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    let principal = selected
        .resolve_entity(
            PrincipalIdentityField::reference(),
            1,
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
    let source = SemanticSource { runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(account),
        output_binding: OutputBindingIdentity::declared("StatusOutput") };
    let upstream = RecordedSettlementIdentity::retain(&source, coordinate, 0);
    let consumer = RecordedSettlementIdentity::retain(&source, coordinate, 1);
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let kind = graph
        .layout()
        .relation(AccountBlocked::reference().name())
        .unwrap()
        .kind;
    let label_ref = AccountLabel::reference();
    let label = graph
        .layout()
        .field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    handle.with_runtime_mut(|runtime| {
        write_field(runtime, account, label.clone(), "prime");
        let (basis_handle, basis) = snapshot(runtime);
        let upstream_facts: Arc<[WorthQueryApplicationObservedFact]> =
            Arc::from([WorthQueryApplicationObservedFact::Relation {
                relation_kind: kind,
                from: principal,
                to: account,
                matching_relations: vec![],
            }]);
        let consumer_facts: Arc<[WorthQueryApplicationObservedFact]> = Arc::from([]);
        for (identity, facts, consumed) in [
            (upstream.clone(), upstream_facts.clone(), OrdSet::new()),
            (
                consumer.clone(),
                consumer_facts.clone(),
                OrdSet::unit(upstream.clone()),
            ),
        ] {
            let mut admission = owner.edit_admission();
            let output = super::super::output_facts::RegisteredOutputFacts {
                facts: Arc::from([WorthQueryApplicationObservedFact::SourceEntity {
                    entity_id: account,
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
                        identity,
                        facts: crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts::for_test(false, facts),
                        output_facts: Some(output),
                        read_basis: basis.clone(),
                        stale_at_read_basis: OrdSet::new(),
                        requirement: None,
                        upstream: consumed,
                    },
                    &mut admission,
                )
                .unwrap();
        }
        let created = CreatedRelationRef {
            partition_id: PartitionId::main(),
            kind_id: kind,
            client_key: ClientKey::raw("pending-recovery"),
            source: EntityReference::Existing(principal),
            target: EntityReference::Existing(account),
        };
        let result = write_batch(
            runtime,
            WorkerIntentBatch::new("add-pending-relation").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: created.partition_id,
                    kind_id: kind,
                    client_key: created.client_key.clone(),
                    source: created.source.clone(),
                    target: created.target.clone(),
                    fields: AspectFieldPatch::default(),
                }),
            )),
        );
        let relation = result.created_relation(&created).unwrap();
        release_test_commit_snapshot(runtime, &result);
        let result = write_batch(
            runtime,
            WorkerIntentBatch::new("remove-pending-relation").push(MutationIntent::Relation(
                RelationMutationIntent::Delete(DeleteRelationIntent {
                    relation_id: relation,
                }),
            )),
        );
        release_test_commit_snapshot(runtime, &result);
        for index in 0..=WINDOW {
            write_field(
                runtime,
                account,
                label.clone(),
                if index % 2 == 0 { "aa" } else { "bb" },
            );
        }
        let (current_handle, current) = snapshot(runtime);
        assert!(matches!(
            currentness(owner, &current, &consumer),
            SourceSettlementCurrentness::FullVerificationRequired(
                super::super::FullVerificationReason::RetainedDeliveryGap
            )
        ));
        // Compare every original source and native output fact before calling
        // the install boundary. Neither row's postconditions supply this proof.
        for fact in upstream_facts
            .iter()
            .chain(consumer_facts.iter())
            .chain(std::iter::once(
                &WorthQueryApplicationObservedFact::SourceEntity { entity_id: account },
            ))
        {
            assert_eq!(
                fact.source_currentness_in(runtime, &current_handle, &mut owner.edit_admission())
                    .unwrap()
                    .unwrap()
                    .movement(),
                crate::domain_computation::primary_graph::application_attempt::Movement::Unmoved
            );
        }
        let establish = |identity: &Arc<_>, facts: &Arc<[WorthQueryApplicationObservedFact]>| {
            owner
                .reestablish_verified(
                    runtime,
                    &current_handle,
                    &current,
                    identity,
                    &crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts::for_test(false, Arc::clone(facts)).for_comparison().unwrap(),
                    &mut owner.edit_admission(),
                )
                .unwrap()
        };
        assert!(
            !establish(&consumer, &consumer_facts),
            "an expired pending row cannot precede its upstream"
        );
        assert!(establish(&upstream, &upstream_facts));
        let cell = owner
            .cell_for_read(&current, &mut owner.edit_admission())
            .unwrap()
            .unwrap();
        assert_eq!(
            cell.read_image()
                .payload()
                .current
                .settlements
                .get(&consumer)
                .unwrap()
                .pending_upstream
                .len(),
            1
        );
        assert!(
            establish(&consumer, &consumer_facts),
            "a full comparison supersedes delivery-only pending marks"
        );
        assert!(matches!(
            currentness(owner, &current, &consumer),
            SourceSettlementCurrentness::Clean
        ));
        assert_eq!(cell.read_image().payload().current.pending_edge_count, 0);
        for snapshot in [basis_handle, current_handle] {
            runtime.snapshots().release_snapshot(&snapshot).unwrap();
        }
    });
}
