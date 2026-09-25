//! Registered descriptors, frozen into admitted capabilities, paired with
//! mounted occurrences, and laid out through the one shared formula.
use std::collections::{BTreeMap, BTreeSet};

use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedInstanceIdentity,
};

use super::{UiNativeMountedComponentLayoutInput, UiNativeMountedRegionLayoutInput};
use crate::capability::{
    ComponentAcceptedRegistrationProof, ComponentAllocationMeasurementContract,
    ComponentChildPolicy, ComponentDescriptor, ComponentId, ComponentPortalChildContract,
    ComponentPropSchema, ComponentStateOwnership, ComponentViewportAxisPlacement as Axis,
    ComponentViewportRegion, FrozenComponentCapabilities, MosaicLayoutCell, MosaicLayoutContract,
    MosaicRegionKindId, MosaicTrack,
};
use crate::runtime::mosaic::layout::UiNativeComponentLayoutDenial;

const LIST_REGION: &str = "demo.region.list";
const BADGE_REGION: &str = "demo.region.badge";
const SHEET_REGION: &str = "demo.region.sheet_body";

fn id(value: &str) -> ComponentId {
    ComponentId::new(value).unwrap()
}

fn component(
    value: &str,
    allocation: ComponentAllocationMeasurementContract,
) -> ComponentDescriptor {
    ComponentDescriptor::new(
        id(value),
        ComponentPropSchema::named(format!("{value}.props")),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
    .with_allocation_measurement_contract(allocation)
}

fn in_cell(
    horizontal: Option<Axis>,
    vertical: Option<Axis>,
) -> ComponentAllocationMeasurementContract {
    ComponentAllocationMeasurementContract::layout_cell(ComponentViewportRegion::new(
        horizontal.unwrap(),
        vertical.unwrap(),
    ))
}

/// A row 100 points in from each side of the viewport, split 3:1 with a
/// 20-point gap, holding a stretched label, a filled card, and an
/// end-anchored icon, and a sheet the row presents as a Portal.
fn descriptors() -> Vec<ComponentDescriptor> {
    let layout = MosaicLayoutContract::columns([
        MosaicTrack::flex(3, 0).unwrap(),
        MosaicTrack::flex(1, 0).unwrap(),
    ])
    .unwrap()
    .with_gaps(20, 0)
    .with_member(id("demo.component.label"), MosaicLayoutCell::at(0, 0))
    .unwrap()
    .with_member(id("demo.component.card"), MosaicLayoutCell::at(1, 0))
    .unwrap()
    .with_member(id("demo.component.icon"), MosaicLayoutCell::at(1, 0))
    .unwrap();
    vec![
        component(
            "demo.component.label",
            in_cell(
                Some(Axis::stretch_between(10, 10)),
                Axis::fixed_from_start(5, 20),
            ),
        ),
        component(
            "demo.component.card",
            ComponentAllocationMeasurementContract::fill_layout_cell(),
        )
        .with_region_allocation(
            MosaicRegionKindId::new(BADGE_REGION).unwrap(),
            ComponentViewportRegion::new(
                Axis::stretch_between(10, 10),
                Axis::stretch_between(10, 10),
            ),
        ),
        component(
            "demo.component.icon",
            in_cell(Axis::fixed_from_end(8, 30), Axis::fixed_from_start(5, 30)),
        ),
        component(
            "demo.component.row",
            ComponentAllocationMeasurementContract::viewport_region(ComponentViewportRegion::new(
                Axis::stretch_between(100, 100),
                Axis::fixed_from_start(50, 80).unwrap(),
            )),
        )
        .with_layout(layout)
        .with_region_allocation(
            MosaicRegionKindId::new(LIST_REGION).unwrap(),
            ComponentViewportRegion::new(
                Axis::stretch_between(100, 100),
                Axis::fixed_from_start(60, 200).unwrap(),
            ),
        ),
        component(
            "demo.component.sheet",
            ComponentAllocationMeasurementContract::viewport_region(ComponentViewportRegion::new(
                Axis::fixed_from_start(300, 200).unwrap(),
                Axis::fixed_from_start(200, 100).unwrap(),
            )),
        )
        .with_portal_child(ComponentPortalChildContract::new(id("demo.component.row")))
        .with_region_allocation(
            MosaicRegionKindId::new(SHEET_REGION).unwrap(),
            ComponentViewportRegion::new(
                Axis::stretch_between(300, 500),
                Axis::fixed_from_start(210, 50).unwrap(),
            ),
        ),
    ]
}

fn host_box(bounds: [f32; 4], coordinate_space: UiMountedCoordinateSpace) -> UiMountedCanonicalBox {
    let [x, y, width, height] = bounds;
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space,
    })
    .unwrap()
}

/// The admitted inputs, in descriptor order, and each component's instance.
fn mounted_inputs() -> (
    Vec<UiNativeMountedComponentLayoutInput>,
    BTreeMap<ComponentId, UiMountedInstanceIdentity>,
) {
    let descriptors = descriptors();
    let accepted = descriptors
        .iter()
        .map(|descriptor| descriptor.id().as_str().to_owned())
        .collect::<BTreeSet<_>>();
    let capabilities = FrozenComponentCapabilities::from_accepted_descriptors(
        descriptors.clone(),
        &ComponentAcceptedRegistrationProof::from_identity_texts(accepted),
    );
    let mounted = descriptors
        .iter()
        .map(|descriptor| {
            (
                descriptor.id().clone(),
                UiMountedInstanceIdentity::mint_unbound().unwrap(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    // Members come before their container, as mounted rows may.
    let inputs = descriptors
        .iter()
        .map(|descriptor| {
            UiNativeMountedComponentLayoutInput::from_descriptor(
                format!("component:{}", descriptor.id().as_str()),
                mounted[descriptor.id()],
                capabilities.get(descriptor.id()),
                &capabilities,
                |component| mounted.get(component).copied(),
            )
        })
        .collect::<Vec<_>>();
    (inputs, mounted)
}

fn viewport() -> UiMountedCanonicalBox {
    host_box(
        [0.0, 0.0, 1_000.0, 600.0],
        UiMountedCoordinateSpace::HostSurface,
    )
}

fn region(owner: UiMountedInstanceIdentity, kind: &str) -> UiNativeMountedRegionLayoutInput {
    UiNativeMountedRegionLayoutInput::from_mounted_region(
        owner,
        kind,
        worth_ui_dsl::UiMosaicRegionDeclarationIdentity::new(7).unwrap(),
        kind,
    )
}

#[test]
fn registered_members_are_placed_within_their_mounted_container() {
    let (inputs, mounted) = mounted_inputs();
    let layout =
        UiNativeMountedComponentLayoutInput::resolve_layout(viewport(), &inputs, &[]).unwrap();
    let occurrences = layout.occurrences();

    let row = mounted[&id("demo.component.row")];
    // The row is 800 wide; 780 remain after the gap, shared 585 and 195.
    let local = UiMountedCoordinateSpace::GraphNodeLocal;
    let expected = [
        (Some(row), host_box([10.0, 5.0, 565.0, 20.0], local)),
        (Some(row), host_box([605.0, 0.0, 195.0, 80.0], local)),
        (Some(row), host_box([762.0, 5.0, 30.0, 30.0], local)),
        (
            None,
            host_box(
                [100.0, 50.0, 800.0, 80.0],
                UiMountedCoordinateSpace::HostSurface,
            ),
        ),
        // A Portal child places against the viewport, relative to its owner.
        (Some(row), host_box([300.0, 200.0, 200.0, 100.0], local)),
    ];
    assert_eq!(occurrences.len(), expected.len());
    for ((occurrence, input), (parent, bounds)) in occurrences.iter().zip(&inputs).zip(expected) {
        assert_eq!(occurrence.instance(), input.instance());
        assert_eq!(
            occurrence.parent(),
            parent,
            "{}",
            input.authored_semantic_identity()
        );
        assert_eq!(
            occurrence.bounds(),
            bounds,
            "{}",
            input.authored_semantic_identity()
        );
    }
}

/// A region resolves within the box its owner's allocation resolved against,
/// then stands local to the owner: a scroll viewport can be taller than the
/// row that owns it, a member's region follows its cell, and a Portal child's
/// region resolves against the viewport it is placed in.
#[test]
fn regions_resolve_within_their_owner_reference_and_stand_local_to_it() {
    let (inputs, mounted) = mounted_inputs();
    let row = mounted[&id("demo.component.row")];
    let card = mounted[&id("demo.component.card")];
    let sheet = mounted[&id("demo.component.sheet")];
    let layout = UiNativeMountedComponentLayoutInput::resolve_layout(
        viewport(),
        &inputs,
        &[
            region(row, LIST_REGION),
            region(card, BADGE_REGION),
            region(sheet, SHEET_REGION),
        ],
    )
    .unwrap();
    let local = UiMountedCoordinateSpace::GraphNodeLocal;
    let bounds = layout
        .regions()
        .iter()
        .map(|region| (region.owner(), region.bounds()))
        .collect::<Vec<_>>();
    assert_eq!(
        bounds,
        [
            (row, host_box([0.0, 10.0, 800.0, 200.0], local)),
            (card, host_box([10.0, 10.0, 175.0, 60.0], local)),
            (sheet, host_box([0.0, 10.0, 200.0, 50.0], local)),
        ]
    );
}

#[test]
fn a_region_its_owner_does_not_place_is_refused() {
    let (inputs, mounted) = mounted_inputs();
    let label = mounted[&id("demo.component.label")];
    assert_eq!(
        UiNativeMountedComponentLayoutInput::resolve_layout(
            viewport(),
            &inputs,
            &[region(label, LIST_REGION)],
        )
        .map(|_| ())
        .unwrap_err(),
        UiNativeComponentLayoutDenial::MissingRegionAllocation
    );
    assert_eq!(
        UiNativeMountedComponentLayoutInput::resolve_layout(
            viewport(),
            &inputs,
            &[region(
                UiMountedInstanceIdentity::mint_unbound().unwrap(),
                LIST_REGION
            )],
        )
        .map(|_| ())
        .unwrap_err(),
        UiNativeComponentLayoutDenial::MissingRegionOwner
    );
}
