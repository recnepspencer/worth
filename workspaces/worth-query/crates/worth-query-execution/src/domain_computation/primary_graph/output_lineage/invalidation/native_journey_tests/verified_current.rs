//! Actor image and retention behavior, using real native source snapshots.
//! The existing test-only equality mint establishes actor coverage only.

use super::*;
use crate::domain_computation::primary_graph::output_lineage::input_cutoff::StableEqualityConsequence;
use crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts;
use worth_relational::facade::mvcc::CompanionCellEditStop;

#[test]
fn repeated_current_certification_preserves_image_and_retention_and_rejects_a_racing_edit() {
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
    let source = SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding: TypeId::of::<StatusOutput>(),
    };
    let prior = RecordedSettlementIdentity::retain(&source, coordinate, 0);
    let current = RecordedSettlementIdentity::retain(&source, coordinate, 1);
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let status_ref = AccountStatus::reference();
    let status = graph
        .layout()
        .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let label_ref = AccountLabel::reference();
    let label = graph
        .layout()
        .field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();

    handle.with_runtime_mut(|runtime| {
        write_field(runtime, entity, label, "prime");
        let (snapshot, basis) = snapshot(runtime);
        let facts: Arc<[_]> = Arc::from([field_fact(runtime, &snapshot, entity, status)]);
        register(
            owner,
            Arc::clone(&prior),
            Arc::clone(&facts),
            &basis,
            OrdSet::new(),
        );
        let relation = StableEqualityConsequence::native_actor_fixture(
            Arc::clone(&prior),
            Arc::clone(&current),
            &basis,
        );
        let alias = owner
            .prepare_current_stable_settlement(
                SettlementRegistration {
                    work_membership: None,
                    identity: Arc::clone(&current),
                    facts: RetainedSourceFacts::for_test(false, Arc::clone(&facts)),
                    output_facts: None,
                    read_basis: basis.clone(),
                    stale_at_read_basis: OrdSet::new(),
                    requirement: None,
                    upstream: OrdSet::new(),
                },
                &basis,
                relation,
                &mut owner.edit_admission(),
            )
            .unwrap();
        drop(
            alias
                .install()
                .unwrap_or_else(|_| panic!("selected alias installs")),
        );
        let cell = owner
            .cell_for_read(&basis, &mut owner.edit_admission())
            .unwrap()
            .unwrap();
        for with_downstream in [false, true] {
            if with_downstream {
                let downstream = RecordedSettlementIdentity::retain(
                    &SemanticSource {
                        output_binding: TypeId::of::<DownstreamOutput>(),
                        ..source.clone()
                    },
                    coordinate,
                    0,
                );
                register(
                    owner,
                    Arc::clone(&downstream),
                    Arc::from([]),
                    &basis,
                    OrdSet::unit(Arc::clone(&current)),
                );
                assert!(matches!(
                    currentness(owner, &basis, &downstream),
                    SourceSettlementCurrentness::Clean
                ));
            }
            let original = cell.read_image();
            let retained = owner.resources.retained_capacity_bytes();
            for _ in 0..3 {
                let prepared = owner
                    .prepare_verified_current(
                        runtime,
                        &snapshot,
                        &basis,
                        &current,
                        &crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts::for_test(false, Arc::clone(&facts)).for_comparison().unwrap(),
                        &mut owner.edit_admission(),
                    )
                    .unwrap()
                    .expect("complete clean actor coverage certifies current");
                drop(
                    prepared
                        .install()
                        .unwrap_or_else(|_| panic!("unchanged image certifies")),
                );
                let observed = cell.read_image();
                assert!(Arc::ptr_eq(original.payload(), observed.payload()));
                assert_eq!(
                    original.topology_generation(),
                    observed.topology_generation()
                );
                assert_eq!(retained, owner.resources.retained_capacity_bytes());
            }
        }

        let original = cell.read_image();
        let stale = owner
            .prepare_verified_current(
                runtime,
                &snapshot,
                &basis,
                &current,
                &crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts::for_test(false, Arc::clone(&facts)).for_comparison().unwrap(),
                &mut owner.edit_admission(),
            )
            .unwrap()
            .unwrap();
        let other = RecordedSettlementIdentity::retain(&source, coordinate, 2);
        register(
            owner,
            Arc::clone(&other),
            Arc::clone(&facts),
            &basis,
            OrdSet::new(),
        );
        let replacement = cell.read_image();
        assert!(!Arc::ptr_eq(original.payload(), replacement.payload()));
        let stopped = match stale.install() {
            Err(stopped) => stopped,
            Ok(_) => panic!("an actor edit must reject the old image certificate"),
        };
        assert_eq!(
            stopped.reason(),
            CompanionCellEditStop::TopologyGenerationChanged
        );
        drop(stopped);
        assert!(Arc::ptr_eq(
            replacement.payload(),
            cell.read_image().payload()
        ));
        assert!(matches!(
            currentness(owner, &basis, &other),
            SourceSettlementCurrentness::Clean
        ));
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
    });
}
