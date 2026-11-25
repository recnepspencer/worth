//! The receiving catalog is the only issuer of coverage for readmitted
//! authored index-write packets (public #384).
use super::*;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::authored_index_writes::CanonicalAuthoredIndexWrites;

fn definition(index_id: u64, name: &str) -> DerivedIndexDefinition {
    DerivedIndexDefinition {
        index_id: DerivedIndexId(index_id),
        name: name.into(),
        branch_scoped: false,
        kind: crate::contracts::indexes::data::definition::DerivedIndexKind::EntityField {
            field_locator:
                crate::contracts::transactions::data::aspect_field_patch::planned_single_field_locator(
                    AspectKey::new("account").unwrap(),
                    FieldKey::new(name).unwrap(),
                ),
        },
    }
}

/// An owner with definitions 1 and 2 installed (next identity 3).
fn owner() -> IndexingSubsystem {
    let owner = IndexingSubsystem::default();
    owner.register_definition(definition(0, "first")).unwrap();
    owner.register_definition(definition(0, "second")).unwrap();
    owner
}

/// A packet as it decodes from durable bytes: a persisted bound, no live proof.
fn persisted(bound: u64) -> CanonicalAuthoredIndexWrites {
    let foreign = IndexCatalogCoverage {
        incarnation: u64::MAX,
        epoch: u64::MAX,
    };
    CanonicalAuthoredIndexWrites::prepared(Vec::new(), None, foreign, bound)
}

/// What `certify_readmitted_writes` issues once its descent is funded: the
/// receiving catalog's coverage before the certified bound.
fn certify(
    catalog: &CapturedIndexDefinitionCatalog,
    writes: &CanonicalAuthoredIndexWrites,
    delta_next_index_id: u64,
) -> Option<IndexCatalogCoverage> {
    catalog
        .readmitted_bound(writes, delta_next_index_id)
        .map(|bound| catalog.catalog.coverage_before(bound))
}

#[test]
fn a_bound_certifies_the_receiving_catalog_prefix_before_it() {
    let catalog = owner().definition_read_binding().capture_catalog();
    let first = catalog.catalog.coverage_before(2);
    let second = catalog.catalog.coverage_before(3);
    assert_eq!(certify(&catalog, &persisted(2), 2), Some(first));
    assert_eq!(certify(&catalog, &persisted(3), 3), Some(second));
    // Coverage before 2 never extends definition 2's introduction.
    assert!(!first.extends(second));
    assert!(second.extends(first));
    // The certified coverage is the receiving catalog's, not the packet's.
    assert_eq!(first.incarnation, catalog.coverage.incarnation);
}

#[test]
fn unavailable_mismatched_and_uninstalled_bounds_certify_nothing() {
    let catalog = owner().definition_read_binding().capture_catalog();
    let unavailable = CanonicalAuthoredIndexWrites::default();
    assert_eq!(certify(&catalog, &unavailable, 3), None);
    assert_eq!(
        certify(&catalog, &persisted(2), 3),
        None,
        "bound disagrees with the delta"
    );
    assert_eq!(
        certify(&catalog, &persisted(3), 2),
        None,
        "bound disagrees with the delta"
    );
    assert_eq!(
        certify(&catalog, &persisted(4), 4),
        None,
        "bound is not installed yet"
    );
}

#[test]
fn certification_from_a_replaced_catalog_never_extends_the_new_incarnation() {
    let owner = owner();
    let before = owner.definition_read_binding().capture_catalog();
    let certified = certify(&before, &persisted(3), 3).unwrap();
    let state = owner.state.read().clone();
    owner.prepare_checkpoint_replacement().install(state);
    let after = owner.definition_read_binding().capture_catalog();
    let renewed = certify(&after, &persisted(3), 3).unwrap();
    assert_ne!(renewed.incarnation, certified.incarnation);
    assert!(!certified.extends(after.catalog.coverage_before(2)));
    assert!(!renewed.extends(certified));
}

#[test]
fn checkpoint_catalog_readmits_only_an_ordered_prefix_into_a_fresh_catalog() {
    let prefix = [definition(1, "first"), definition(2, "second")];
    let mut fresh = IndexingState::default();
    fresh.readmit_checkpoint_catalog(&prefix, 3).unwrap();
    assert_eq!(fresh.next_index_id, 3);

    let refused = "checkpoint index catalog readmits only into a fresh catalog";
    let mut populated = IndexingState::default();
    populated
        .readmit_checkpoint_catalog(&prefix[..1], 2)
        .unwrap();
    assert_eq!(
        populated.readmit_checkpoint_catalog(&prefix, 3),
        Err(refused)
    );
    let mut moved = IndexingState::default();
    moved.next_index_id = 2;
    assert_eq!(moved.readmit_checkpoint_catalog(&prefix, 3), Err(refused));

    let mut out_of_order = IndexingState::default();
    let reversed = [definition(2, "second"), definition(1, "first")];
    assert_eq!(
        out_of_order.readmit_checkpoint_catalog(&reversed, 3),
        Err("checkpoint index catalog is not a complete ordered prefix")
    );
}
