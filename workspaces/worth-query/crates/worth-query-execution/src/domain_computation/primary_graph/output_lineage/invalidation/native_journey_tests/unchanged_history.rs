//! Same-position native replacements share their unchanged history allocation.
use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;

use super::*;
use crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity;
use crate::domain_computation::primary_graph::output_lineage::invalidation::source_alignment::BranchMarkRoot;
use std::alloc::Layout;

#[test]
fn pinned_same_position_replacements_share_history_but_own_their_root_bytes() {
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
        output_binding: OutputBindingIdentity::declared("StatusOutput"),
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
    let label_ref = AccountLabel::reference();
    let label = graph
        .layout()
        .field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    handle.with_runtime_mut(|runtime| {
        for ordinal in 0..20 {
            write_field(
                runtime,
                entity,
                label.clone(),
                &format!("history-{ordinal}"),
            );
        }
        let (basis_handle, basis) = snapshot(runtime);
        let cell = owner
            .cell_for_read(&basis, &mut owner.edit_admission())
            .unwrap()
            .unwrap();
        let original = cell.read_image();
        assert!(!original.payload().past.is_empty());
        let history = Arc::clone(original.payload().history_capacity.as_ref().unwrap());
        let historical_sources = original
            .payload()
            .past
            .iter()
            .map(|(position, row)| (*position, row.root_id, row.commit_id))
            .collect::<Vec<_>>();
        let expected_root_charge = arc_layout_bytes::<BranchMarkRoot>()
            + arc_layout_bytes::<RetainedInvalidationCapacity>();
        assert_eq!(
            original
                .payload()
                .retained_capacity
                .as_ref()
                .unwrap()
                .bytes(),
            expected_root_charge,
            "the root object charge excludes the separately owned history map"
        );
        let facts = Arc::from([field_fact(runtime, &basis_handle, entity, status)]);
        let mut images = vec![original];
        let mut identities = Vec::new();
        for ordinal in 0..8 {
            let identity = RecordedSettlementIdentity::retain(&source, coordinate, ordinal);
            register(
                owner,
                Arc::clone(&identity),
                Arc::clone(&facts),
                &basis,
                OrdSet::new(),
            );
            assert!(matches!(
                currentness(owner, &basis, &identity),
                SourceSettlementCurrentness::Clean
            ));
            let image = cell.read_image();
            assert_eq!(image.root_id(), basis.root_id());
            assert_eq!(image.commit_id(), basis.commit_id());
            assert_eq!(image.position(), basis.position());
            assert!(Arc::ptr_eq(
                image.payload().history_capacity.as_ref().unwrap(),
                &history
            ));
            let root_charge = image.payload().retained_capacity.as_ref().unwrap();
            assert_eq!(root_charge.bytes(), expected_root_charge);
            assert!(images.iter().all(|old| !Arc::ptr_eq(
                old.payload().retained_capacity.as_ref().unwrap(),
                root_charge
            )));
            assert_eq!(
                image
                    .payload()
                    .past
                    .iter()
                    .map(|(position, row)| (*position, row.root_id, row.commit_id))
                    .collect::<Vec<_>>(),
                historical_sources
            );
            images.push(image);
            identities.push(identity);
        }
        let pinned = owner.resources.retained_capacity_bytes();
        // The live cell still holds the final replacement. Releasing old native
        // image leases reclaims their objects without releasing its shared map.
        images.clear();
        assert!(owner.resources.retained_capacity_bytes() < pinned);
        assert!(Arc::ptr_eq(
            cell.read_image()
                .payload()
                .history_capacity
                .as_ref()
                .unwrap(),
            &history
        ));
        let last = cell.read_image();
        write_field(runtime, entity, label, "edited-history");
        let next = cell.read_image();
        assert_ne!(next.position(), last.position());
        assert!(
            !Arc::ptr_eq(next.payload().history_capacity.as_ref().unwrap(), &history),
            "a native history edit retains its own conservative map charge"
        );
        assert_eq!(last.payload().past.len(), historical_sources.len());
        for identity in identities {
            assert!(
                matches!(
                    currentness(owner, &basis, &identity),
                    SourceSettlementCurrentness::Clean
                ),
                "a later publication preserves the exact retained read basis"
            );
        }
        let pinned = owner.resources.retained_capacity_bytes();
        drop(history);
        drop(last);
        assert!(owner.resources.retained_capacity_bytes() < pinned);
        runtime.snapshots().release_snapshot(&basis_handle).unwrap();
    });
}

// Independent layout oracle for an Arc allocation: two atomic counters followed
// by the correctly aligned payload. No production accounting helper is called.
fn arc_layout_bytes<T>() -> u64 {
    Layout::new::<[usize; 2]>()
        .extend(Layout::new::<T>())
        .unwrap()
        .0
        .pad_to_align()
        .size() as u64
}
