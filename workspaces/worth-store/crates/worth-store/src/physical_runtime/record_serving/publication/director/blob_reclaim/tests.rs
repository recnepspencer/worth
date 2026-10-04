//! No reclaim drop resolves a route that is not an extent, and none resolves
//! a SessionDeclared record whatever the planner selected.

use worth_store_physical_format::{
    BlobRecordKind, CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement, ExtentArenaId,
    ExtentArenaRange, PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration,
    PhysicalGenerationAuthority, SelectedRecordContentClass, SelectedRecordRouteMetadata,
};

use super::droppable_extent;
use crate::physical_runtime::durability::PhysicalBlobReclaimAdmissionDenial as Denial;

fn routed(kind: BlobRecordKind) -> DurableExtentRecordPlacement {
    DurableExtentRecordPlacement::new_selected(
        PersistedRecordIdentity::new([7; 16], 9).unwrap(),
        PhysicalGenerationAuthority::for_canonical_physical_format()
            .record_extent_cell(PhysicalExtentId::from_raw(3).unwrap())
            .with_extent_generation(PhysicalGeneration::from_raw(4).unwrap()),
        4096,
        ExtentArenaRange::new(ExtentArenaId::new(2).unwrap(), 53_248, 53_248).unwrap(),
        SelectedRecordRouteMetadata::primary(SelectedRecordContentClass::Blob(kind)).unwrap(),
    )
    .unwrap()
}

#[test]
fn a_drop_set_naming_a_declaration_record_is_denied_before_any_fence() {
    let declaration = routed(BlobRecordKind::SessionDeclared);
    assert_eq!(
        droppable_extent(Some(CurrentPhysicalRecordPlacement::Extent(declaration))),
        Err(Denial::SelectedResidueInvalid)
    );
}

#[test]
fn the_records_a_release_does_drop_resolve_to_their_extents() {
    for kind in [
        BlobRecordKind::Chunk,
        BlobRecordKind::TreeNode,
        BlobRecordKind::GenerationPublished,
    ] {
        let extent = routed(kind);
        assert_eq!(
            droppable_extent(Some(CurrentPhysicalRecordPlacement::Extent(extent))),
            Ok(extent)
        );
    }
    assert_eq!(droppable_extent(None), Err(Denial::RouteUnavailable));
}
