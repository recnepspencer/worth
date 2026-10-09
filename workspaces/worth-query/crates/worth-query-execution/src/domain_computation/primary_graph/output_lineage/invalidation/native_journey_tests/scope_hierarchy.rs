//! A Native leaf touch queues its ancestor scopes and no sibling subtree.
use super::super::logical_marking::NativeMarkingPrecision;
use super::*;
use crate::domain_computation::primary_graph::{
    application_output_demand::{SelectedRequiredWorkKind, WorthQueryOutputDemandRegistry},
    tests::fixture::{Account, AccountIdentity},
};
use worth_relational::facade::{
    identity::PartitionId,
    symbols::ClientKey,
    transactions::{CreateIntent, CreatedEntityRef, EntitySpec},
};
const DEPTH: usize = 6;
struct ScopeOutput;

#[test]
fn a_deep_native_leaf_has_identical_work_with_eight_or_sixty_four_sibling_subtrees() {
    let mut counts = None;
    for siblings in [8, 64] {
        let observed = run(siblings);
        assert_eq!(
            *counts.get_or_insert(observed),
            observed,
            "the leaf's logical work depends on its ancestor path, not the sibling population"
        );
    }
}

fn run(siblings: usize) -> super::super::logical_marking::LogicalMarkingCounts {
    let world = installed_authorization_world(true);
    let (_, product, _) = world.selected_product().into_parts();
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
        graph
            .layout()
            .field_locator(entity, aspect, field)
            .unwrap()
            .clone()
    };
    let i = AccountIdentity::reference();
    let s = AccountStatus::reference();
    let l = AccountLabel::reference();
    let identity = locator(i.entity(), i.aspect(), i.field());
    let status = locator(s.entity(), s.aspect(), s.field());
    let label = locator(l.entity(), l.aspect(), l.field());
    handle.with_runtime_mut(|runtime| {
        let refs: Vec<_> = (0..(siblings + 1) * DEPTH).map(|n| CreatedEntityRef {
            partition_id: PartitionId::main(), kind_id: kind,
            client_key: ClientKey::raw(format!("scope-tree-{n}")),
        }).collect();
        let batch = refs.iter().enumerate().fold(WorkerIntentBatch::new("scope-forest"), |batch, (n, r)| {
            let value = |text| AspectValue::String(InternedString::Raw(text));
            batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: r.partition_id, kind_id: r.kind_id, client_key: r.client_key.clone(),
                fields: AspectFieldPatch::from(BTreeMap::from([
                    (identity.clone(), value(format!("scope-tree-{n}"))),
                    (status.clone(), value("neutral".to_owned())),
                    (label.clone(), value("before".to_owned())),
                ])),
            })))
        });
        let committed = write_batch(runtime, batch);
        let entities: Vec<_> = refs.iter().map(|r| committed.created_entity(r).unwrap()).collect();
        release_test_commit_snapshot(runtime, &committed);
        let (before_handle, before) = snapshot(runtime);
        let registries: Vec<_> = entities.iter().map(|_| WorthQueryOutputDemandRegistry::default()).collect();
        let members: Vec<_> = registries.iter().zip(&entities).map(|(registry, entity)|
            registry.fixture_scope_work_membership(coordinate.occurrence, *entity)).collect();
        let outputs: Vec<_> = entities.iter().map(|entity| RecordedSettlementIdentity::retain(&SemanticSource {
            runtime_authority: world.application.runtime.authority_identity().as_u64(),
            schema: world.application.installed_schema.binding_identity().clone(),
            scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(*entity),
            output_binding: TypeId::of::<ScopeOutput>(),
        }, coordinate, 0)).collect();
        for n in 0..entities.len() {
            // Each distinct Native scope above the leaf consumes exactly its
            // child's output. Sibling paths have no consumed edge in common.
            let leaf = n % DEPTH == 0;
            owner.register_settlement(SettlementRegistration {
                work_membership: Some(Arc::clone(&members[n].1)),
                identity: Arc::clone(&outputs[n]),
                facts: crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts::for_test(false,
                    if leaf { Arc::from([field_fact(runtime, &before_handle, entities[n], label.clone())]) } else { Arc::from([]) }),
                output_facts: None, read_basis: before.clone(), stale_at_read_basis: OrdSet::new(), requirement: None,
                upstream: if leaf { OrdSet::new() } else { OrdSet::unit(Arc::clone(&outputs[n - 1])) },
            }, &mut owner.edit_admission()).unwrap();
            // Admission queues initial disclosure; registration queues one
            // local settlement cue. Both precede the measured Native edit.
            for _ in 0..2 {
                let initial = registries[n].next_required_work(owner, &mut owner.edit_admission()).unwrap().unwrap();
                assert!(matches!(initial.kind(), SelectedRequiredWorkKind::Local { .. } | SelectedRequiredWorkKind::UnresolvedInitial));
                initial.acknowledge();
            }
            assert!(registries[n].next_required_work(owner, &mut owner.edit_admission()).unwrap().is_none());
        }
        write_field(runtime, entities[0], label, "after");
        let (after_handle, after) = snapshot(runtime);
        let report = owner.native_marking_report(&after, &mut owner.edit_admission()).unwrap().unwrap();
        let NativeMarkingPrecision::Exact(counts) = report.precision else { panic!("the declared Native leaf touch is exact") };
        assert_eq!(counts.marked_fact_ordinals, 1);
        assert_eq!(counts.visited_vertices, DEPTH as u64);
        assert_eq!(counts.downstream_edges, DEPTH as u64 - 1);
        for (n, registry) in registries.iter().enumerate() {
            let candidate = registry.next_required_work(owner, &mut owner.edit_admission()).unwrap();
            if n < DEPTH {
                let candidate = candidate.expect("each ancestor has one Native pending cue");
                assert!(matches!(candidate.kind(), SelectedRequiredWorkKind::Native { .. }));
                candidate.acknowledge();
            } else {
                assert!(candidate.is_none(), "sibling scope {n} contributes zero candidates, hence zero Ready selections");
                assert!(matches!(currentness(owner, &after, &outputs[n]), SourceSettlementCurrentness::Clean));
            }
            assert!(registry.next_required_work(owner, &mut owner.edit_admission()).unwrap().is_none(), "one cue per dirty scope");
        }
        for snapshot in [before_handle, after_handle] { runtime.snapshots().release_snapshot(&snapshot).unwrap(); }
        counts
    })
}
