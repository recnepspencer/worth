use worth_ui_host_contract::UiMountedInstanceIdentity;

use super::{
    resolve_component_layout, UiMosaicLayoutBox, UiMosaicLayoutNode, UiMosaicLayoutParent,
    UiNativeComponentLayoutDenial,
};
use crate::capability::{
    ComponentAllocationMeasurementContract, ComponentId, ComponentViewportAxisPlacement,
    ComponentViewportRegion, MosaicLayoutCell, MosaicLayoutContract, MosaicResponsiveLayout,
    MosaicTrack, MosaicViewportWidthInterval,
};

fn instance() -> UiMountedInstanceIdentity {
    UiMountedInstanceIdentity::mint_unbound().unwrap()
}

fn id(value: &str) -> ComponentId {
    ComponentId::new(value).unwrap()
}

fn card_row(members: &[ComponentId]) -> MosaicResponsiveLayout {
    let columns = |count: usize| {
        let tracks = (0..count).map(|_| MosaicTrack::flex(1, 0).unwrap());
        MosaicLayoutContract::columns(tracks)
            .unwrap()
            .with_gaps(20, 20)
    };
    let grid = |layout: MosaicLayoutContract, per_row: u16| {
        members
            .iter()
            .enumerate()
            .fold(layout, |layout, (index, member)| {
                let index = index as u16;
                let cell = MosaicLayoutCell::at(index % per_row, index / per_row);
                layout.with_member(member.clone(), cell).unwrap()
            })
    };
    let two_by_two = MosaicLayoutContract::grid(
        [
            MosaicTrack::flex(1, 0).unwrap(),
            MosaicTrack::flex(1, 0).unwrap(),
        ],
        [
            MosaicTrack::flex(1, 0).unwrap(),
            MosaicTrack::flex(1, 0).unwrap(),
        ],
    )
    .unwrap()
    .with_gaps(20, 20);
    MosaicResponsiveLayout::new(grid(two_by_two, 2))
        .with_variant(
            MosaicViewportWidthInterval::at_least(1200),
            grid(columns(4), 4),
        )
        .unwrap()
}

fn row_region() -> ComponentAllocationMeasurementContract {
    ComponentAllocationMeasurementContract::viewport_region(ComponentViewportRegion::new(
        ComponentViewportAxisPlacement::stretch_between(260, 24),
        ComponentViewportAxisPlacement::fixed_from_start(104, 120).unwrap(),
    ))
}

#[test]
fn members_follow_the_container_across_viewport_widths_and_variants() {
    let members = [
        id("demo.component.a"),
        id("demo.component.b"),
        id("demo.component.c"),
        id("demo.component.d"),
    ];
    let layout = card_row(&members);
    let row = instance();
    let cards = members.each_ref().map(|_| instance());
    let mut nodes = vec![UiMosaicLayoutNode {
        instance: row,
        allocation: Some(row_region()),
        layout: Some(&layout),
        parent: UiMosaicLayoutParent::Viewport,
    }];
    for (card, member) in cards.iter().zip(&members) {
        nodes.push(UiMosaicLayoutNode {
            instance: *card,
            allocation: Some(ComponentAllocationMeasurementContract::fill_layout_cell()),
            layout: None,
            parent: UiMosaicLayoutParent::Container {
                container: row,
                member,
            },
        });
    }

    // 1364 wide: the row spans 1080, so four columns of 255 with 20 gaps.
    let wide = resolve_component_layout(1364.0, 900.0, &nodes).unwrap();
    assert_eq!(wide[0].parent, None);
    assert_eq!(
        wide[0].bounds,
        UiMosaicLayoutBox {
            x: 260.0,
            y: 104.0,
            width: 1080.0,
            height: 120.0
        }
    );
    assert_eq!(wide[3].parent, Some(row));
    assert_eq!(
        wide[3].bounds,
        UiMosaicLayoutBox {
            x: 550.0,
            y: 0.0,
            width: 255.0,
            height: 120.0
        }
    );

    // 1024 wide selects the 2x2 fallback: 740 across, 50 per row.
    let narrow = resolve_component_layout(1024.0, 900.0, &nodes).unwrap();
    assert_eq!(
        narrow[4].bounds,
        UiMosaicLayoutBox {
            x: 380.0,
            y: 70.0,
            width: 360.0,
            height: 50.0
        }
    );
}

#[test]
fn a_member_region_is_placed_within_its_cell() {
    let member = id("demo.component.label");
    let layout = MosaicResponsiveLayout::from(
        MosaicLayoutContract::columns([
            MosaicTrack::fixed(100).unwrap(),
            MosaicTrack::flex(1, 0).unwrap(),
        ])
        .unwrap()
        .with_member(member.clone(), MosaicLayoutCell::at(1, 0))
        .unwrap(),
    );
    let container = instance();
    let label = instance();
    let nodes = [
        UiMosaicLayoutNode {
            instance: label,
            allocation: Some(ComponentAllocationMeasurementContract::layout_cell(
                ComponentViewportRegion::new(
                    ComponentViewportAxisPlacement::stretch_between(8, 8),
                    ComponentViewportAxisPlacement::fixed_from_end(4, 20).unwrap(),
                ),
            )),
            layout: None,
            parent: UiMosaicLayoutParent::Container {
                container,
                member: &member,
            },
        },
        UiMosaicLayoutNode {
            instance: container,
            allocation: Some(ComponentAllocationMeasurementContract::fill_viewport()),
            layout: Some(&layout),
            parent: UiMosaicLayoutParent::Viewport,
        },
    ];
    let placed = resolve_component_layout(400.0, 60.0, &nodes).unwrap();
    assert_eq!(
        placed[0].bounds,
        UiMosaicLayoutBox {
            x: 108.0,
            y: 36.0,
            width: 284.0,
            height: 20.0
        }
    );
}

#[test]
fn unplaceable_members_are_denied_instead_of_guessed() {
    let member = id("demo.component.card");
    let orphan = [UiMosaicLayoutNode {
        instance: instance(),
        allocation: Some(ComponentAllocationMeasurementContract::fill_layout_cell()),
        layout: None,
        parent: UiMosaicLayoutParent::Viewport,
    }];
    assert_eq!(
        resolve_component_layout(800.0, 600.0, &orphan),
        Err(UiNativeComponentLayoutDenial::MissingLayoutContainer)
    );
    let unmounted = [UiMosaicLayoutNode {
        instance: instance(),
        allocation: Some(ComponentAllocationMeasurementContract::fill_layout_cell()),
        layout: None,
        parent: UiMosaicLayoutParent::Container {
            container: instance(),
            member: &member,
        },
    }];
    assert_eq!(
        resolve_component_layout(800.0, 600.0, &unmounted),
        Err(UiNativeComponentLayoutDenial::MissingLayoutContainer)
    );
    let first = instance();
    let second = instance();
    let layout = MosaicResponsiveLayout::from(
        MosaicLayoutContract::frame()
            .unwrap()
            .with_member(member.clone(), MosaicLayoutCell::at(0, 0))
            .unwrap(),
    );
    let cycle = [first, second].map(|node| UiMosaicLayoutNode {
        instance: node,
        allocation: Some(ComponentAllocationMeasurementContract::fill_layout_cell()),
        layout: Some(&layout),
        parent: UiMosaicLayoutParent::Container {
            container: if node == first { second } else { first },
            member: &member,
        },
    });
    assert_eq!(
        resolve_component_layout(800.0, 600.0, &cycle),
        Err(UiNativeComponentLayoutDenial::ContainerCycle)
    );
}
