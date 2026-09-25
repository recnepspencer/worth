use std::path::PathBuf;

use worth_ui_dsl::{WorthUiAuthoredSourceInput, WorthUiDslCompiler, WorthUiSealedSemanticPackage};

use crate::capability::{
    ComponentAllocationMeasurementContract, ComponentChildPolicy, ComponentDescriptor, ComponentId,
    ComponentPropSchema, ComponentStateOwnership, ComponentViewportAxisPlacement,
    ComponentViewportRegion, MosaicLayoutCell, MosaicLayoutContract, MosaicLayoutDenial,
    MosaicResponsiveLayout, MosaicTrack, MosaicViewportWidthInterval, UiAuthoredLayoutCause,
};
use crate::facade::entry::WorthUiHostNeutralApp;
use crate::facade::WorthUi;

use super::{prepare_semantic_handoff, WorthUiSemanticHandoffPreparationStop};

const ROW: &str = "test.component.row";
const LABEL: &str = "test.component.label";
const ICON: &str = "test.component.icon";

/// The narrow grid and the wide variant the fixture row registers, spelled
/// as a `layout` block.
const RESTATED: &str = "
    columns flex 1;
    rows fixed 40, fixed 40;
    gap 0 8;
    padding 12 6;
    member test.component.label at 0 0;
    member test.component.icon at 0 1;
    width from 800 {
      columns flex 3 min 200, fixed 48;
      rows fixed 40;
      gap 16 0;
      padding 12 6;
      member test.component.label at 0 0;
      member test.component.icon at 1 0;
    }";

/// A block that states the registered layout is agreement: it admits, and
/// the snapshot is not rebuilt.
#[test]
fn a_layout_that_restates_the_registered_layout_admits() {
    let app = row_app();
    let material = prepare_semantic_handoff(layout_package(ROW, RESTATED), app.capabilities())
        .expect("a layout block that restates its container's layout admits");
    let (_, _, evidence) = material.into_parts();

    assert_eq!(evidence.successor_snapshot(), None);
}

/// A layout's members are a set: the order they are listed in states
/// nothing.
#[test]
fn member_order_does_not_change_the_layout() {
    let swap = |source: &str, first: &str, second: &str| {
        let swapped = source.replace(
            &format!("{first}\n{second}"),
            &format!("{}\n{}", second.trim_start(), first),
        );
        assert_ne!(
            swapped, source,
            "the fixture holds `{first}` before `{second}`"
        );
        swapped
    };
    let reordered = swap(
        RESTATED,
        "    member test.component.label at 0 0;",
        "    member test.component.icon at 0 1;",
    );
    let reordered = swap(
        &reordered,
        "      member test.component.label at 0 0;",
        "      member test.component.icon at 1 0;",
    );

    prepare_semantic_handoff(layout_package(ROW, &reordered), row_app().capabilities())
        .expect("the same members in another order are the same layout");
}

/// Two sources naming one layout with different numbers disagree, and
/// neither the authored grid nor the registered one wins.
#[test]
fn a_layout_that_states_another_layout_is_denied() {
    let wider_gap = RESTATED.replace("gap 16 0;", "gap 24 0;");

    assert_eq!(
        refused(layout_package(ROW, &wider_gap), 0),
        UiAuthoredLayoutCause::LayoutDisagreement
    );
}

/// A variant the registration does not have is a different layout too.
#[test]
fn a_missing_variant_is_a_disagreement() {
    let narrow_only = &RESTATED[..RESTATED
        .find("width from")
        .expect("the fixture has a variant")];

    assert_eq!(
        refused(layout_package(ROW, narrow_only), 0),
        UiAuthoredLayoutCause::LayoutDisagreement
    );
}

#[test]
fn a_layout_cannot_name_an_unregistered_container() {
    assert_eq!(
        refused(layout_package("test.component.absent", RESTATED), 0),
        UiAuthoredLayoutCause::UnregisteredContainer
    );
}

#[test]
fn a_layout_cannot_name_a_component_that_lays_nothing_out() {
    assert_eq!(
        refused(layout_package(LABEL, RESTATED), 0),
        UiAuthoredLayoutCause::ContainerHasNoLayout
    );
}

/// The component identity grammar is the registry's; a spelling the
/// language admits can still name nothing it could hold.
#[test]
fn a_layout_names_well_formed_component_identities() {
    assert_eq!(
        refused(layout_package("Test.Row", RESTATED), 0),
        UiAuthoredLayoutCause::ContainerIdentityMalformed
    );
    assert_eq!(
        refused(
            layout_package(ROW, &RESTATED.replace("icon at 1 0;", "Icon at 1 0;")),
            0
        ),
        UiAuthoredLayoutCause::MemberIdentityMalformed
    );
}

/// One container, one layout block: a second is refused even when it agrees.
#[test]
fn a_second_layout_for_one_container_is_denied() {
    let package = compile(format!(
        "layout {ROW} {{{RESTATED}}}\nlayout {ROW} {{{RESTATED}}}"
    ));

    assert_eq!(
        refused(package, 1),
        UiAuthoredLayoutCause::DuplicateDeclaration
    );
}

/// The Mosaic constructors judge the authored grid; the language only
/// spells it.
#[test]
fn a_track_without_weight_is_judged_by_the_mosaic_owner() {
    let zero = RESTATED.replace("columns flex 1;", "columns flex 0;");

    assert_eq!(
        refused(layout_package(ROW, &zero), 0),
        UiAuthoredLayoutCause::Layout(MosaicLayoutDenial::ZeroWeight)
    );
}

#[test]
fn a_cell_past_the_tracks_is_judged_by_the_mosaic_owner() {
    let outside = RESTATED.replace("icon at 1 0;", "icon at 2 0;");

    assert_eq!(
        refused(layout_package(ROW, &outside), 0),
        UiAuthoredLayoutCause::Layout(MosaicLayoutDenial::CellOutsideTracks)
    );
}

#[test]
fn overlapping_width_variants_are_judged_by_the_mosaic_owner() {
    let variant = &RESTATED[RESTATED
        .find("width from")
        .expect("the fixture has a variant")..];
    let overlapping = format!("{RESTATED}\n{}", variant.replace("800", "900"));

    assert_eq!(
        refused(layout_package(ROW, &overlapping), 0),
        UiAuthoredLayoutCause::Layout(MosaicLayoutDenial::OverlappingViewportIntervals)
    );
}

#[test]
fn a_variant_with_other_members_is_judged_by_the_mosaic_owner() {
    let missing = RESTATED.replace("      member test.component.icon at 1 0;", "");

    assert_eq!(
        refused(layout_package(ROW, &missing), 0),
        UiAuthoredLayoutCause::Layout(MosaicLayoutDenial::VariantMembershipMismatch)
    );
}

fn refused(package: WorthUiSealedSemanticPackage, index: usize) -> UiAuthoredLayoutCause {
    let denial = match prepare_semantic_handoff(package, row_app().capabilities()) {
        Ok(_) => panic!("the authored layout must not admit"),
        Err(denial) => denial,
    };
    let WorthUiSemanticHandoffPreparationStop::AuthoredLayout(refusal) = denial.stop() else {
        panic!("the stop must name the authored layout that was refused")
    };
    assert_eq!(refusal.declaration_index(), index);
    refusal.cause()
}

fn layout_package(container: &str, body: &str) -> WorthUiSealedSemanticPackage {
    compile(format!("layout {container} {{{body}}}"))
}

fn compile(source: String) -> WorthUiSealedSemanticPackage {
    WorthUiDslCompiler::compile_source(
        WorthUiAuthoredSourceInput::rooted_at(PathBuf::from("workspace"))
            .with_module("app/main.wui", source),
    )
    .expect("layout source seals")
}

fn id(value: &str) -> ComponentId {
    ComponentId::new(value).expect("the fixture component id is well formed")
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

fn registered_layout() -> MosaicResponsiveLayout {
    let narrow = MosaicLayoutContract::grid(
        [MosaicTrack::flex(1, 0).unwrap()],
        [
            MosaicTrack::fixed(40).unwrap(),
            MosaicTrack::fixed(40).unwrap(),
        ],
    )
    .unwrap()
    .with_gaps(0, 8)
    .with_padding(12, 6)
    .with_member(id(LABEL), MosaicLayoutCell::at(0, 0))
    .unwrap()
    .with_member(id(ICON), MosaicLayoutCell::at(0, 1))
    .unwrap();
    let wide = MosaicLayoutContract::grid(
        [
            MosaicTrack::flex(3, 200).unwrap(),
            MosaicTrack::fixed(48).unwrap(),
        ],
        [MosaicTrack::fixed(40).unwrap()],
    )
    .unwrap()
    .with_gaps(16, 0)
    .with_padding(12, 6)
    .with_member(id(LABEL), MosaicLayoutCell::at(0, 0))
    .unwrap()
    .with_member(id(ICON), MosaicLayoutCell::at(1, 0))
    .unwrap();
    MosaicResponsiveLayout::new(narrow)
        .with_variant(MosaicViewportWidthInterval::at_least(800), wide)
        .unwrap()
}

fn row_app() -> WorthUiHostNeutralApp {
    let row =
        ComponentAllocationMeasurementContract::viewport_region(ComponentViewportRegion::new(
            ComponentViewportAxisPlacement::stretch_between(0, 0),
            ComponentViewportAxisPlacement::fixed_from_start(0, 92).unwrap(),
        ));
    WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_component(component(ROW, row).with_layout(registered_layout()))
        .register_component(component(
            LABEL,
            ComponentAllocationMeasurementContract::fill_layout_cell(),
        ))
        .register_component(component(
            ICON,
            ComponentAllocationMeasurementContract::fill_layout_cell(),
        ))
        .freeze()
        .expect("the layout fixture app freezes")
}
